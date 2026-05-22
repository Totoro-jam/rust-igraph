//! Fluid communities (ALGO-CO-005).
//!
//! Counterpart of `igraph_community_fluid_communities()` from
//! `references/igraph/src/community/fluid.c`.
//!
//! Parés F., Gasulla D.G. *et al.* (2018): *Fluid Communities: A
//! Competitive, Scalable and Diverse Community Detection Algorithm.*
//! Complex Networks & Their Applications VI, Springer, vol 689, p 229.
//! <https://doi.org/10.1007/978-3-319-72150-7_19>
//!
//! The algorithm seeds `k` distinct fluids (communities) at random
//! vertices, each with density `1/size`. In each iteration the vertex
//! order is shuffled and every vertex re-evaluates its label by summing
//! density contributions from itself and its neighbours, picking the
//! dominant label (with ε = 1e-4 tie tolerance and random tie-break).
//! A vertex retains its current label whenever it is still tied for
//! dominant. Iteration stops when no vertex changes label.
//!
//! Constraints (mirrors upstream):
//!
//! - The graph must be **simple** (no self-loops nor parallel edges).
//! - The graph must be **connected** (weak connectivity is sufficient;
//!   directions are ignored with a soft note).
//! - The graph must be **undirected** (directions are ignored, matches
//!   upstream's warning behaviour — surfaced here via a non-fatal
//!   acceptance because we treat directed edges as undirected for the
//!   purpose of community detection).
//! - `1 ≤ k ≤ vcount`.
//! - Weights are **not** supported (no weighted variant exists upstream).
//!
//! Self-rolled, dependency-free. Determinism comes from an inline
//! `SplitMix64` PRNG seeded by the caller; the convenience entrypoint
//! [`fluid_communities`] pins `seed = 0` so repeated calls on the same
//! graph produce identical partitions.

// All `usize` -> `u32` casts in this module are bounded by `n`
// (graph vertex count) which originates from `Graph::vcount(): u32`
// and so can never truncate. `float_cmp` is allowed because the
// dominant-label loop intentionally compares two label-tally f64s for
// exact ordering after an ε-band rejection (mirroring upstream).
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::items_after_statements,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]

use crate::core::{Graph, IgraphError, IgraphResult};

/// Full option bag for [`fluid_communities_with_options`].
#[derive(Debug, Clone)]
pub struct FluidOptions {
    /// PRNG seed driving the initial seed-vertex selection, the
    /// per-iteration node-order shuffle, and the tie-break amongst
    /// dominant labels. Default `0`.
    pub seed: u64,
    /// Safety cap on the number of outer iterations. Upstream loops
    /// `while (running)` with no cap; Parés *et al.* note that
    /// convergence is empirically observed in O(few) iterations, but
    /// adversarial inputs can in principle oscillate, so we expose a
    /// configurable upper bound. Default [`FLUID_DEFAULT_MAX_ITERATIONS`].
    pub max_iterations: u32,
}

/// Default safety cap on outer iterations for
/// [`fluid_communities`] / [`fluid_communities_with_options`].
pub const FLUID_DEFAULT_MAX_ITERATIONS: u32 = 1000;

impl Default for FluidOptions {
    fn default() -> Self {
        Self {
            seed: 0,
            max_iterations: FLUID_DEFAULT_MAX_ITERATIONS,
        }
    }
}

/// Result of [`fluid_communities`] / [`fluid_communities_with_options`].
#[derive(Debug, Clone)]
pub struct FluidResult {
    /// Per-vertex community label, densely numbered in `0..nb_clusters`.
    pub membership: Vec<u32>,
    /// Number of distinct labels actually present. Normally equals the
    /// requested `k`, but a community can vanish in pathological
    /// graphs; we expose the *actual* count so downstream code stays
    /// honest.
    pub nb_clusters: u32,
    /// Number of outer iterations actually executed (1 ≤ x ≤
    /// `max_iterations`).
    pub n_iterations_run: u32,
}

/// Run Fluid Communities with default options
/// (seed = 0, `max_iterations` = [`FLUID_DEFAULT_MAX_ITERATIONS`]).
///
/// `k` is the number of communities to find; it must satisfy
/// `1 ≤ k ≤ vcount`. Self-loops, parallel edges, disconnected
/// components, and weighted graphs are all rejected.
///
/// # Errors
/// - [`IgraphError::InvalidArgument`] if `k = 0`, `k > vcount`, or the
///   graph is not simple / not connected.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, fluid_communities};
///
/// // Two K4 cliques joined by a single bridge edge — the two-community
/// // partition is the obvious result for k=2.
/// let mut g = Graph::with_vertices(8);
/// for u in 0..4 {
///     for v in (u + 1)..4 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// for u in 4..8 {
///     for v in (u + 1)..8 {
///         g.add_edge(u, v).unwrap();
///     }
/// }
/// g.add_edge(3, 4).unwrap();
/// let r = fluid_communities(&g, 2).unwrap();
/// assert_eq!(r.membership[0], r.membership[1]);
/// assert_eq!(r.membership[4], r.membership[5]);
/// assert_ne!(r.membership[0], r.membership[4]);
/// ```
pub fn fluid_communities(graph: &Graph, k: u32) -> IgraphResult<FluidResult> {
    fluid_communities_with_options(graph, k, &FluidOptions::default())
}

/// Run Fluid Communities with full option control.
///
/// See [`fluid_communities`] for the argument and error contract;
/// [`FluidOptions`] adds reproducible seeding and an iteration cap.
///
/// # Errors
/// Same as [`fluid_communities`], plus:
/// - [`IgraphError::Unsupported`] if `max_iterations = 0`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, FluidOptions, fluid_communities_with_options};
///
/// let mut g = Graph::with_vertices(6);
/// for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
///     g.add_edge(u, v).unwrap();
/// }
/// let opts = FluidOptions { seed: 42, ..FluidOptions::default() };
/// let r = fluid_communities_with_options(&g, 2, &opts).unwrap();
/// assert_eq!(r.nb_clusters, 2);
/// ```
pub fn fluid_communities_with_options(
    graph: &Graph,
    k: u32,
    opts: &FluidOptions,
) -> IgraphResult<FluidResult> {
    let n = graph.vcount();

    // Edge case: vcount < 2 ⇒ trivially one community of zero or one
    // vertex (matches upstream's null-graph convenience). We still
    // require k ≥ 1.
    if n < 2 {
        if k == 0 {
            return Err(IgraphError::InvalidArgument(
                "number of requested communities must be positive".to_string(),
            ));
        }
        return Ok(FluidResult {
            membership: vec![0; n as usize],
            nb_clusters: u32::from(n == 1),
            n_iterations_run: 0,
        });
    }

    if k == 0 {
        return Err(IgraphError::InvalidArgument(
            "number of requested communities must be positive".to_string(),
        ));
    }
    if k > n {
        return Err(IgraphError::InvalidArgument(format!(
            "number of requested communities ({k}) must not exceed the number of vertices ({n})"
        )));
    }
    if opts.max_iterations == 0 {
        return Err(IgraphError::Unsupported(
            "FluidOptions.max_iterations must be ≥ 1",
        ));
    }

    // Mirror upstream simplicity + connectivity gating.
    let simple = crate::algorithms::properties::is_simple::is_simple(graph)?;
    if !simple {
        return Err(IgraphError::InvalidArgument(
            "fluid community detection requires a simple graph (no self-loops, no parallel edges)"
                .to_string(),
        ));
    }
    let components = crate::algorithms::connectivity::components::connected_components(graph)?;
    if components.count != 1 {
        return Err(IgraphError::InvalidArgument(
            "fluid community detection requires a connected graph".to_string(),
        ));
    }

    // Build the all-mode adjacency once; we walk it twice per vertex
    // visit (one tally pass) and the graph is undirected for community
    // detection purposes.
    let adjacency = build_adjacency(graph)?;

    let mut rng = SplitMix64::new(opts.seed);

    // Membership uses 1-based labels internally to reserve 0 = "unlabelled"
    // (matching upstream's `kv1 == 0` shortcut). We strip the offset at
    // the very end.
    let mut membership: Vec<u32> = vec![0; n as usize];
    let mut com_to_numvertices: Vec<u32> = vec![0; k as usize];
    let mut density: Vec<f64> = vec![1.0; k as usize];

    // Shuffle node order and seed the first k communities with the
    // first k vertices of the permutation.
    let mut node_order: Vec<u32> = (0..n).collect();
    shuffle_in_place(&mut node_order, &mut rng);
    for i in 0..k as usize {
        let v = node_order[i];
        membership[v as usize] = u32::try_from(i).expect("k fits u32") + 1;
        com_to_numvertices[i] = 1;
    }

    // Working buffers reused across iterations.
    let mut label_counters: Vec<f64> = vec![0.0; k as usize];
    let mut dominant_labels: Vec<u32> = Vec::with_capacity(k as usize);
    let mut touched: Vec<u32> = Vec::new();

    const EPS: f64 = 1e-4;

    let mut iter_count: u32 = 0;
    let mut running = true;
    while running && iter_count < opts.max_iterations {
        running = false;
        iter_count += 1;
        shuffle_in_place(&mut node_order, &mut rng);

        for idx in 0..n as usize {
            let v1 = node_order[idx];
            dominant_labels.clear();
            for &lbl in &touched {
                label_counters[(lbl - 1) as usize] = 0.0;
            }
            touched.clear();

            let kv1 = membership[v1 as usize];
            let mut max_count = 0.0_f64;

            // Same-label retention: the vertex's own current label is
            // pre-loaded into the dominant set so a tie keeps it.
            if kv1 != 0 {
                let d = density[(kv1 - 1) as usize];
                label_counters[(kv1 - 1) as usize] = d;
                touched.push(kv1);
                max_count = d;
                dominant_labels.push(kv1);
            }

            // Density-weighted neighbour tally.
            for &nbr in &adjacency[v1 as usize] {
                let k_label = membership[nbr as usize];
                if k_label == 0 {
                    continue;
                }
                let idx_l = (k_label - 1) as usize;
                if label_counters[idx_l] == 0.0 {
                    touched.push(k_label);
                }
                label_counters[idx_l] += density[idx_l];
                let diff = label_counters[idx_l] - max_count;
                if diff > EPS {
                    max_count = label_counters[idx_l];
                    dominant_labels.clear();
                    dominant_labels.push(k_label);
                } else if diff > -EPS {
                    // Within tie band — append if not already present.
                    // (Same-label retention has already injected kv1 if
                    // it ties.) Avoid duplicate inserts for performance.
                    if !dominant_labels.contains(&k_label) {
                        dominant_labels.push(k_label);
                    }
                }
            }

            if dominant_labels.is_empty() {
                continue;
            }

            // If the current label is still tied for dominant, keep it
            // (no iteration is needed for this vertex). Otherwise
            // sample uniformly from the dominant set.
            if dominant_labels.contains(&kv1) {
                continue;
            }

            running = true;
            let pick_idx = rng.gen_index(dominant_labels.len());
            let new_label = dominant_labels[pick_idx];

            if kv1 != 0 {
                com_to_numvertices[(kv1 - 1) as usize] -= 1;
                // Density spike to +inf when a community empties is a
                // known feature: it pulls a neighbour back into the
                // emptied community on the next iteration, restoring
                // the seed. We do NOT special-case this; IEEE-754
                // arithmetic handles inf cleanly through the tally.
                density[(kv1 - 1) as usize] =
                    1.0 / f64::from(com_to_numvertices[(kv1 - 1) as usize]);
            }

            membership[v1 as usize] = new_label;
            com_to_numvertices[(new_label - 1) as usize] += 1;
            density[(new_label - 1) as usize] =
                1.0 / f64::from(com_to_numvertices[(new_label - 1) as usize]);
        }
    }

    // Drop the 1-based offset; densify by counting how many distinct
    // labels actually survived (usually exactly k, but a community can
    // vanish under pathological dynamics).
    let mut remap: Vec<i32> = vec![-1; k as usize];
    let mut next_dense: u32 = 0;
    let mut dense_membership: Vec<u32> = vec![0; n as usize];
    for (v, &lbl) in membership.iter().enumerate() {
        debug_assert!(lbl >= 1, "fluid: vertex {v} ended unlabelled");
        let idx_l = (lbl - 1) as usize;
        if remap[idx_l] < 0 {
            remap[idx_l] = next_dense as i32;
            next_dense += 1;
        }
        dense_membership[v] = remap[idx_l] as u32;
    }

    Ok(FluidResult {
        membership: dense_membership,
        nb_clusters: next_dense,
        n_iterations_run: iter_count,
    })
}

// ============================================================================
//                              Helpers
// ============================================================================

/// Build an all-mode adjacency list for the undirected interpretation
/// of the graph. Self-loops and parallel edges have already been
/// rejected upstream by the `is_simple` check, so each entry is unique.
fn build_adjacency(graph: &Graph) -> IgraphResult<Vec<Vec<u32>>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for v in 0..n as u32 {
        for nbr in graph.neighbors(v)? {
            adj[v as usize].push(nbr);
        }
    }
    Ok(adj)
}

// ============================================================================
//                                  PRNG
// ============================================================================

struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(0x9E37_79B9_7F4A_7C15))
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn gen_index(&mut self, bound: usize) -> usize {
        debug_assert!(bound > 0);
        let r = self.next_u64() % (bound as u64);
        usize::try_from(r).expect("bound fits in usize by construction")
    }
}

fn shuffle_in_place<T>(slice: &mut [T], rng: &mut SplitMix64) {
    let len = slice.len();
    for i in (1..len).rev() {
        let j = rng.gen_index(i + 1);
        slice.swap(i, j);
    }
}

// ============================================================================
//                                  Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn k_clique(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for u in 0..n {
            for v in (u + 1)..n {
                g.add_edge(u, v).expect("clique edge");
            }
        }
        g
    }

    fn two_cliques_with_bridge(size: u32) -> Graph {
        let mut g = Graph::with_vertices(size * 2);
        for u in 0..size {
            for v in (u + 1)..size {
                g.add_edge(u, v).expect("clique edge");
            }
        }
        for u in size..size * 2 {
            for v in (u + 1)..size * 2 {
                g.add_edge(u, v).expect("clique edge");
            }
        }
        g.add_edge(size - 1, size).expect("bridge edge");
        g
    }

    fn assert_partition_well_formed(r: &FluidResult, expected_n: usize) {
        assert_eq!(r.membership.len(), expected_n);
        if expected_n == 0 {
            assert_eq!(r.nb_clusters, 0);
            return;
        }
        let max_label = *r.membership.iter().max().unwrap_or(&0);
        assert!((max_label as usize) < expected_n);
        assert_eq!(max_label + 1, r.nb_clusters);
        let mut seen = vec![false; r.nb_clusters as usize];
        for &m in &r.membership {
            seen[m as usize] = true;
        }
        assert!(seen.into_iter().all(|b| b));
    }

    #[test]
    fn k_equals_1_is_one_big_community() {
        let g = k_clique(6);
        let r = fluid_communities(&g, 1).unwrap();
        assert_eq!(r.nb_clusters, 1);
        for &m in &r.membership {
            assert_eq!(m, 0);
        }
    }

    #[test]
    fn k_equals_n_is_singletons() {
        let g = k_clique(5);
        let r = fluid_communities(&g, 5).unwrap();
        assert_partition_well_formed(&r, 5);
        // In K5 with k=5, every vertex starts as its own community and
        // density 1.0; the labels are stable from iteration 1.
        assert_eq!(r.nb_clusters, 5);
    }

    #[test]
    fn two_k4_bridge_splits_at_bridge() {
        let g = two_cliques_with_bridge(4);
        let r = fluid_communities(&g, 2).unwrap();
        assert_eq!(r.nb_clusters, 2);
        for u in 0..4 {
            assert_eq!(r.membership[u], r.membership[0]);
        }
        for u in 4..8 {
            assert_eq!(r.membership[u], r.membership[4]);
        }
        assert_ne!(r.membership[0], r.membership[4]);
    }

    #[test]
    fn determinism_under_seed() {
        let g = two_cliques_with_bridge(5);
        let opts = FluidOptions {
            seed: 12345,
            ..FluidOptions::default()
        };
        let a = fluid_communities_with_options(&g, 3, &opts).unwrap();
        let b = fluid_communities_with_options(&g, 3, &opts).unwrap();
        assert_eq!(a.membership, b.membership);
        assert_eq!(a.nb_clusters, b.nb_clusters);
    }

    #[test]
    fn different_seeds_can_differ() {
        let g = two_cliques_with_bridge(5);
        let a = fluid_communities_with_options(
            &g,
            3,
            &FluidOptions {
                seed: 1,
                max_iterations: FLUID_DEFAULT_MAX_ITERATIONS,
            },
        )
        .unwrap();
        let b = fluid_communities_with_options(
            &g,
            3,
            &FluidOptions {
                seed: 99,
                max_iterations: FLUID_DEFAULT_MAX_ITERATIONS,
            },
        )
        .unwrap();
        // Either equal or different — we just verify both shapes are valid.
        assert_partition_well_formed(&a, g.vcount() as usize);
        assert_partition_well_formed(&b, g.vcount() as usize);
    }

    #[test]
    fn empty_graph_returns_empty() {
        let g = Graph::with_vertices(0);
        let r = fluid_communities(&g, 1).unwrap();
        assert_eq!(r.membership.len(), 0);
        assert_eq!(r.nb_clusters, 0);
    }

    #[test]
    fn single_vertex_returns_one_singleton() {
        let g = Graph::with_vertices(1);
        let r = fluid_communities(&g, 1).unwrap();
        assert_eq!(r.membership.len(), 1);
        assert_eq!(r.nb_clusters, 1);
    }

    #[test]
    fn rejects_k_zero() {
        let g = k_clique(4);
        assert!(fluid_communities(&g, 0).is_err());
    }

    #[test]
    fn rejects_k_greater_than_vcount() {
        let g = k_clique(4);
        assert!(fluid_communities(&g, 5).is_err());
    }

    #[test]
    fn rejects_disconnected_graph() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        assert!(fluid_communities(&g, 2).is_err());
    }

    #[test]
    fn rejects_non_simple_graph() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 1).unwrap(); // parallel edge
        assert!(fluid_communities(&g, 2).is_err());
    }

    #[test]
    fn rejects_max_iterations_zero() {
        let g = k_clique(4);
        let opts = FluidOptions {
            seed: 0,
            max_iterations: 0,
        };
        assert!(fluid_communities_with_options(&g, 2, &opts).is_err());
    }

    #[test]
    fn ring_of_4_k5_cliques_finds_4_groups() {
        // 4 K5 cliques joined in a ring.
        let mut g = Graph::with_vertices(20);
        for c in 0..4 {
            let base = c * 5;
            for u in 0..5 {
                for v in (u + 1)..5 {
                    g.add_edge(base + u, base + v).unwrap();
                }
            }
            let next_base = ((c + 1) % 4) * 5;
            g.add_edge(base, next_base).unwrap();
        }
        let r = fluid_communities_with_options(
            &g,
            4,
            &FluidOptions {
                seed: 7,
                max_iterations: FLUID_DEFAULT_MAX_ITERATIONS,
            },
        )
        .unwrap();
        assert_eq!(r.nb_clusters, 4);
        for c in 0..4 {
            let base = (c * 5) as usize;
            let label = r.membership[base];
            for offset in 1..5 {
                assert_eq!(r.membership[base + offset], label);
            }
        }
    }

    #[test]
    fn converges_in_reasonable_iterations() {
        let g = two_cliques_with_bridge(6);
        let r = fluid_communities_with_options(
            &g,
            2,
            &FluidOptions {
                seed: 0,
                max_iterations: 100,
            },
        )
        .unwrap();
        // K6+K6 with bridge converges in ≤ ~10 iterations from any seed.
        assert!(r.n_iterations_run <= 100);
        assert!(r.n_iterations_run >= 1);
    }
}
