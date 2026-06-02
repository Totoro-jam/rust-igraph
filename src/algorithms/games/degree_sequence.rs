//! Configuration-model degree-sequence generator (ALGO-GN-024).
//!
//! Counterpart of the `IGRAPH_DEGSEQ_CONFIGURATION` branch of
//! `igraph_degree_sequence_game()` in
//! `references/igraph/src/games/degree_sequence.c` (function
//! `configuration`, lines 37-123).
//!
//! The configuration model (Bender–Canfield 1978 / Bollobás 1980) builds
//! a random graph that realises a given degree sequence by
//! **stub matching**:
//!
//! 1. Expand each vertex `i` into `d_i` half-edge "stubs" — a flat bag of
//!    `Σ d_i` stubs.
//! 2. Repeatedly pick two stubs uniformly at random and pair them into an
//!    edge.
//! 3. Project the matched stubs to a graph by reading off vertex labels.
//!
//! The result is a **multigraph** — self-loops and multi-edges are
//! allowed and occur with positive probability whenever the degree
//! sequence permits them. To sample uniformly from *simple* graphs with a
//! given degree sequence, use the Viger–Latapy (`VL`) or
//! `EDGE_SWITCHING_SIMPLE` methods instead (planned for follow-up AWUs
//! GN-025+).
//!
//! ## Directed vs undirected
//!
//! * **Undirected** (`in_degrees = None`): single bag of size
//!   `outsum = Σ out_degrees`. Each iteration pops two stubs (with
//!   replacement-via-swap-pop) and emits a single edge. Requires
//!   `outsum` to be **even** so the bag fully drains.
//! * **Directed** (`in_degrees = Some(in_seq)`): two bags. Out-stubs are
//!   the edge sources, in-stubs the sinks. Requires
//!   `Σ out_degrees == Σ in_degrees`.
//!
//! ## Determinism
//!
//! All randomness flows from a single `SplitMix64` seed — the same
//! `(out_degrees, in_degrees, seed)` triple always yields the same graph.
//! The PRNG state is not bitwise portable to igraph C / `NumPy` / R, so
//! the three-source conformance harness asserts structural invariants
//! only (vcount, ecount, exact degree match, directed-ness).
//!
//! ## Complexity
//!
//! Stub-bag construction: `Θ(n + Σ d_i)`. Edge sampling: `Θ(|E|)` with
//! `O(1)` random-access pops via Fisher-Yates–style swap-removal.
//! Total: `Θ(n + |E|)` time and `Θ(|E|)` peak memory for the bags.

#![allow(clippy::cast_possible_truncation)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Sum a slice of `u32` degrees into `u64` with overflow checking.
fn checked_sum(degrees: &[u32]) -> IgraphResult<u64> {
    let mut acc: u64 = 0;
    for &d in degrees {
        acc = acc
            .checked_add(u64::from(d))
            .ok_or(IgraphError::Internal("degree-sum overflow"))?;
    }
    Ok(acc)
}

/// Expand a degree slice into a flat stub bag where vertex `i` appears
/// exactly `degrees[i]` times.
fn build_bag(degrees: &[u32], total: u64) -> IgraphResult<Vec<VertexId>> {
    let cap = usize::try_from(total)
        .map_err(|_| IgraphError::Internal("degree-sequence stub count exceeds usize"))?;
    let mut bag: Vec<VertexId> = Vec::with_capacity(cap);
    for (i, &d) in degrees.iter().enumerate() {
        let vid = VertexId::try_from(i)
            .map_err(|_| IgraphError::Internal("degree-sequence vertex index exceeds u32"))?;
        for _ in 0..d {
            bag.push(vid);
        }
    }
    Ok(bag)
}

/// Sample a random graph realising the given degree sequence(s) via the
/// configuration / stub-matching model.
///
/// * `out_degrees` — degree of every vertex (undirected) or out-degree
///   of every vertex (directed). Length defines vertex count `n`.
/// * `in_degrees` — when `Some(seq)`, switches to directed mode and
///   `seq[i]` is the in-degree of vertex `i`. Must have the same length
///   as `out_degrees` and the same total.
/// * `seed` — drives the internal `SplitMix64` PRNG.
///
/// The output is a **multigraph**: self-loops and parallel edges can
/// occur whenever the degree sequence allows them, with the natural
/// configuration-model probabilities.
///
/// # Errors
///
/// * `in_degrees` length disagrees with `out_degrees` length.
/// * Degree sum overflows `u64` (only possible at billion-vertex scale).
/// * Undirected mode with an odd `Σ out_degrees` (not graphical).
/// * Directed mode with `Σ out_degrees != Σ in_degrees`.
/// * Edge count overflows `u32` (igraph's hard `ECOUNT_MAX`).
/// * Vertex count exceeds `u32::MAX`.
///
/// # Examples
///
/// ```
/// use rust_igraph::degree_sequence_game_configuration;
///
/// // Undirected 4-cycle target: every vertex degree 2 ⇒ 4 edges total.
/// let g = degree_sequence_game_configuration(&[2, 2, 2, 2], None, 7).unwrap();
/// assert_eq!(g.vcount(), 4);
/// assert_eq!(g.ecount(), 4);
/// assert!(!g.is_directed());
/// ```
pub fn degree_sequence_game_configuration(
    out_degrees: &[u32],
    in_degrees: Option<&[u32]>,
    seed: u64,
) -> IgraphResult<Graph> {
    let directed = in_degrees.is_some();
    let n_usize = out_degrees.len();
    let n = u32::try_from(n_usize)
        .map_err(|_| IgraphError::Internal("degree-sequence vertex count exceeds u32"))?;

    if let Some(in_seq) = in_degrees {
        if in_seq.len() != n_usize {
            return Err(IgraphError::InvalidArgument(format!(
                "degree_sequence_game_configuration: out_degrees has length {} but in_degrees has length {}",
                n_usize,
                in_seq.len()
            )));
        }
    }

    if n == 0 {
        return Graph::new(0, directed);
    }

    let outsum = checked_sum(out_degrees)?;
    let (insum, no_of_edges) = if let Some(in_seq) = in_degrees {
        let insum = checked_sum(in_seq)?;
        if outsum != insum {
            return Err(IgraphError::InvalidArgument(format!(
                "degree_sequence_game_configuration: out-degree sum {outsum} != in-degree sum {insum} (no directed graph realises the given sequences)"
            )));
        }
        (insum, outsum)
    } else {
        if outsum % 2 != 0 {
            return Err(IgraphError::InvalidArgument(format!(
                "degree_sequence_game_configuration: undirected degree sum {outsum} is odd (no graph realises an odd-sum degree sequence)"
            )));
        }
        (0u64, outsum / 2)
    };

    if no_of_edges > u64::from(u32::MAX) {
        return Err(IgraphError::Internal(
            "degree_sequence_game_configuration: edge count exceeds u32::MAX",
        ));
    }
    let no_of_edges_usize = no_of_edges as usize;

    let mut bag1 = build_bag(out_degrees, outsum)?;
    let mut bag2_opt: Option<Vec<VertexId>> = if let Some(in_seq) = in_degrees {
        Some(build_bag(in_seq, insum)?)
    } else {
        None
    };

    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(no_of_edges_usize);
    let mut rng = SplitMix64::new(seed);

    if let Some(bag2) = bag2_opt.as_mut() {
        for _ in 0..no_of_edges_usize {
            let from_idx = rng.gen_index(bag1.len());
            let to_idx = rng.gen_index(bag2.len());
            let from = bag1.swap_remove(from_idx);
            let to = bag2.swap_remove(to_idx);
            edges.push((from, to));
        }
    } else {
        for _ in 0..no_of_edges_usize {
            let from_idx = rng.gen_index(bag1.len());
            let from = bag1.swap_remove(from_idx);
            // Bag has at least one element left because outsum is even.
            let to_idx = rng.gen_index(bag1.len());
            let to = bag1.swap_remove(to_idx);
            edges.push((from, to));
        }
    }

    let mut g = Graph::new(n, directed)?;
    g.add_edges(edges)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observed_undirected_degrees(graph: &Graph) -> Vec<u32> {
        let vcount = graph.vcount() as usize;
        let mut deg = vec![0u32; vcount];
        let ecount = u32::try_from(graph.ecount()).expect("test ecount fits u32");
        for eid in 0..ecount {
            let (src, dst) = graph.edge(eid).expect("test edge id in bounds");
            deg[src as usize] = deg[src as usize].saturating_add(1);
            deg[dst as usize] = deg[dst as usize].saturating_add(1);
        }
        deg
    }

    fn observed_directed_degrees(graph: &Graph) -> (Vec<u32>, Vec<u32>) {
        let vcount = graph.vcount() as usize;
        let mut outd = vec![0u32; vcount];
        let mut ind = vec![0u32; vcount];
        let ecount = u32::try_from(graph.ecount()).expect("test ecount fits u32");
        for eid in 0..ecount {
            let (src, dst) = graph.edge(eid).expect("test edge id in bounds");
            outd[src as usize] = outd[src as usize].saturating_add(1);
            ind[dst as usize] = ind[dst as usize].saturating_add(1);
        }
        (outd, ind)
    }

    #[test]
    fn empty_degree_sequence_returns_empty_graph() {
        let g = degree_sequence_game_configuration(&[], None, 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(!g.is_directed());
    }

    #[test]
    fn empty_directed_degree_sequence_returns_empty_graph() {
        let g = degree_sequence_game_configuration(&[], Some(&[]), 0).unwrap();
        assert_eq!(g.vcount(), 0);
        assert_eq!(g.ecount(), 0);
        assert!(g.is_directed());
    }

    #[test]
    fn all_zero_undirected_produces_edgeless_graph() {
        let g = degree_sequence_game_configuration(&[0, 0, 0, 0, 0], None, 1).unwrap();
        assert_eq!(g.vcount(), 5);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn all_zero_directed_produces_edgeless_graph() {
        let g = degree_sequence_game_configuration(&[0, 0, 0], Some(&[0, 0, 0]), 1).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 0);
    }

    #[test]
    fn observed_undirected_degrees_match_input() {
        let seq = [2, 3, 2, 3, 3, 3, 3, 1, 4, 4];
        let g = degree_sequence_game_configuration(&seq, None, 333).unwrap();
        assert_eq!(g.vcount(), seq.len() as u32);
        let observed = observed_undirected_degrees(&g);
        assert_eq!(observed, seq);
    }

    #[test]
    fn observed_directed_degrees_match_input() {
        let out_seq = [2, 3, 2, 3, 3, 3, 3, 1, 4, 4];
        let in_seq = [3, 6, 2, 0, 2, 2, 4, 3, 3, 3];
        let g = degree_sequence_game_configuration(&out_seq, Some(&in_seq), 333).unwrap();
        assert_eq!(g.vcount(), out_seq.len() as u32);
        let (outd, ind) = observed_directed_degrees(&g);
        assert_eq!(outd, out_seq);
        assert_eq!(ind, in_seq);
    }

    #[test]
    fn ecount_equals_half_undirected_sum() {
        let seq = [4u32, 4, 4, 4, 4, 4, 4, 4]; // sum = 32, expect 16 edges
        let g = degree_sequence_game_configuration(&seq, None, 17).unwrap();
        assert_eq!(g.ecount(), 16);
    }

    #[test]
    fn ecount_equals_outsum_directed() {
        let out_seq = [2u32, 1, 3];
        let in_seq = [1u32, 3, 2];
        let g = degree_sequence_game_configuration(&out_seq, Some(&in_seq), 11).unwrap();
        assert_eq!(g.ecount(), 6);
    }

    #[test]
    fn rejects_length_mismatch() {
        let err = degree_sequence_game_configuration(&[1, 2, 3], Some(&[1, 2]), 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("length")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_odd_undirected_sum() {
        let err = degree_sequence_game_configuration(&[1, 1, 1], None, 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => assert!(msg.contains("odd")),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_directed_sum_mismatch() {
        let err = degree_sequence_game_configuration(&[1, 2, 3], Some(&[1, 1, 1]), 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(msg) => {
                assert!(msg.contains("out-degree sum") && msg.contains("in-degree sum"));
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn deterministic_same_seed_same_edges() {
        let seq = [3u32, 3, 3, 3, 3, 3];
        let g1 = degree_sequence_game_configuration(&seq, None, 42).unwrap();
        let g2 = degree_sequence_game_configuration(&seq, None, 42).unwrap();
        let collect = |g: &Graph| -> Vec<(VertexId, VertexId)> {
            let m = u32::try_from(g.ecount()).unwrap();
            let mut v: Vec<_> = (0..m)
                .map(|e| {
                    let (u, w) = g.edge(e).unwrap();
                    if u <= w { (u, w) } else { (w, u) }
                })
                .collect();
            v.sort_unstable();
            v
        };
        assert_eq!(collect(&g1), collect(&g2));
    }

    #[test]
    fn distinct_seeds_usually_differ() {
        let seq = [4u32, 4, 4, 4, 4, 4, 4, 4];
        let g1 = degree_sequence_game_configuration(&seq, None, 1).unwrap();
        let g2 = degree_sequence_game_configuration(&seq, None, 2).unwrap();
        let collect = |g: &Graph| -> Vec<(VertexId, VertexId)> {
            let m = u32::try_from(g.ecount()).unwrap();
            let mut v: Vec<_> = (0..m)
                .map(|e| {
                    let (u, w) = g.edge(e).unwrap();
                    if u <= w { (u, w) } else { (w, u) }
                })
                .collect();
            v.sort_unstable();
            v
        };
        assert_ne!(collect(&g1), collect(&g2));
    }

    #[test]
    fn single_vertex_with_self_loop_via_d2() {
        // Only realisation of [2] is one self-loop.
        let g = degree_sequence_game_configuration(&[2], None, 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 1);
        let (u, v) = g.edge(0).unwrap();
        assert_eq!((u, v), (0, 0));
    }

    #[test]
    fn directed_single_vertex_self_loop() {
        // d_out=[1], d_in=[1] ⇒ one self-loop.
        let g = degree_sequence_game_configuration(&[1], Some(&[1]), 1).unwrap();
        assert_eq!(g.vcount(), 1);
        assert_eq!(g.ecount(), 1);
        let (u, v) = g.edge(0).unwrap();
        assert_eq!((u, v), (0, 0));
    }

    #[test]
    fn directed_path_realisation() {
        // d_out=[1,1,0], d_in=[0,1,1] — the only realisation is 0→1, 1→2.
        let g = degree_sequence_game_configuration(&[1, 1, 0], Some(&[0, 1, 1]), 1).unwrap();
        assert_eq!(g.vcount(), 3);
        assert_eq!(g.ecount(), 2);
        let edges: Vec<_> = (0..2).map(|e| g.edge(e).unwrap()).collect();
        let mut got = edges.clone();
        got.sort_unstable();
        assert_eq!(got, vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn determinism_across_sweep() {
        // Run a sweep of (n, seed) pairs and confirm reproducibility.
        for n in [3u32, 8, 20, 50] {
            let seq: Vec<u32> = (0..n).map(|i| 2 + (i % 3)).collect();
            // Ensure even sum.
            let mut seq = seq;
            let s: u32 = seq.iter().sum();
            if s % 2 != 0 {
                seq[0] += 1;
            }
            for seed in [0u64, 1, 7, 1_234_567] {
                let g1 = degree_sequence_game_configuration(&seq, None, seed).unwrap();
                let g2 = degree_sequence_game_configuration(&seq, None, seed).unwrap();
                assert_eq!(g1.vcount(), g2.vcount());
                assert_eq!(g1.ecount(), g2.ecount());
                assert_eq!(
                    observed_undirected_degrees(&g1),
                    observed_undirected_degrees(&g2)
                );
            }
        }
    }

    #[test]
    fn all_ones_undirected_is_perfect_matching_size() {
        let seq = [1u32; 10];
        let g = degree_sequence_game_configuration(&seq, None, 99).unwrap();
        assert_eq!(g.ecount(), 5);
        let observed = observed_undirected_degrees(&g);
        assert_eq!(observed, seq);
    }

    #[test]
    fn large_uniform_directed_passes_degree_check() {
        let n = 100u32;
        let d = 5u32;
        let out_seq = vec![d; n as usize];
        let in_seq = vec![d; n as usize];
        let g = degree_sequence_game_configuration(&out_seq, Some(&in_seq), 0xABCD).unwrap();
        assert_eq!(g.vcount(), n);
        assert_eq!(g.ecount(), (n * d) as usize);
        let (outd, ind) = observed_directed_degrees(&g);
        assert_eq!(outd, out_seq);
        assert_eq!(ind, in_seq);
    }

    #[test]
    fn directed_zero_in_zero_out_disconnected_vertex_allowed() {
        // Vertex 2 has degree 0 in both directions — must remain isolated.
        let out_seq = [1u32, 1, 0];
        let in_seq = [1u32, 1, 0];
        let g = degree_sequence_game_configuration(&out_seq, Some(&in_seq), 0).unwrap();
        let (outd, ind) = observed_directed_degrees(&g);
        assert_eq!(outd, out_seq);
        assert_eq!(ind, in_seq);
    }

    #[test]
    fn rejects_directed_with_out_zero_in_nonzero() {
        // Sum mismatch: 0 vs 2.
        let err = degree_sequence_game_configuration(&[0, 0], Some(&[1, 1]), 1).unwrap_err();
        match err {
            IgraphError::InvalidArgument(_) => {}
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn even_sum_seq() -> impl Strategy<Value = Vec<u32>> {
        prop::collection::vec(0u32..6, 0..15).prop_map(|mut v| {
            let s: u32 = v.iter().sum();
            if s % 2 != 0 && !v.is_empty() {
                v[0] += 1;
            }
            v
        })
    }

    fn matched_directed_seqs() -> impl Strategy<Value = (Vec<u32>, Vec<u32>)> {
        prop::collection::vec((0u32..5, 0u32..5), 0..12).prop_map(|pairs| {
            let mut out: Vec<u32> = pairs.iter().map(|p| p.0).collect();
            let mut inn: Vec<u32> = pairs.iter().map(|p| p.1).collect();
            let os: u64 = out.iter().map(|&d| u64::from(d)).sum();
            let is: u64 = inn.iter().map(|&d| u64::from(d)).sum();
            if !out.is_empty() {
                if os > is {
                    inn[0] = inn[0].saturating_add(u32::try_from(os - is).unwrap_or(0));
                } else if is > os {
                    out[0] = out[0].saturating_add(u32::try_from(is - os).unwrap_or(0));
                }
            }
            (out, inn)
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn undirected_degree_match(seq in even_sum_seq(), seed in any::<u64>()) {
            let g = degree_sequence_game_configuration(&seq, None, seed)
                .expect("valid input must succeed");
            prop_assert_eq!(g.vcount(), seq.len() as u32);
            let sum: u64 = seq.iter().map(|&d| u64::from(d)).sum();
            prop_assert_eq!(g.ecount() as u64, sum / 2);
            let n = u32::try_from(g.vcount()).unwrap();
            let mut deg = vec![0u32; n as usize];
            let m = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m {
                let (u, v) = g.edge(eid).unwrap();
                deg[u as usize] += 1;
                deg[v as usize] += 1;
            }
            prop_assert_eq!(deg, seq);
        }

        #[test]
        fn directed_degree_match((out_seq, in_seq) in matched_directed_seqs(), seed in any::<u64>()) {
            let g = degree_sequence_game_configuration(&out_seq, Some(&in_seq), seed)
                .expect("valid input must succeed");
            prop_assert_eq!(g.vcount(), out_seq.len() as u32);
            let n = u32::try_from(g.vcount()).unwrap();
            let mut outd = vec![0u32; n as usize];
            let mut ind = vec![0u32; n as usize];
            let m = u32::try_from(g.ecount()).unwrap();
            for eid in 0..m {
                let (u, v) = g.edge(eid).unwrap();
                outd[u as usize] += 1;
                ind[v as usize] += 1;
            }
            prop_assert_eq!(outd, out_seq);
            prop_assert_eq!(ind, in_seq);
        }

        #[test]
        fn deterministic_same_seed(seq in even_sum_seq(), seed in any::<u64>()) {
            let g1 = degree_sequence_game_configuration(&seq, None, seed)
                .expect("valid input must succeed");
            let g2 = degree_sequence_game_configuration(&seq, None, seed)
                .expect("valid input must succeed");
            let collect = |g: &Graph| -> Vec<(VertexId, VertexId)> {
                let m = u32::try_from(g.ecount()).unwrap();
                let mut v: Vec<_> = (0..m).map(|e| g.edge(e).unwrap()).collect();
                v.sort_unstable();
                v
            };
            prop_assert_eq!(collect(&g1), collect(&g2));
        }

        #[test]
        fn odd_sum_rejected(seq in prop::collection::vec(0u32..6, 1..15)) {
            let s: u32 = seq.iter().sum();
            prop_assume!(s % 2 != 0);
            let r = degree_sequence_game_configuration(&seq, None, 0);
            prop_assert!(r.is_err());
        }
    }
}
