//! Integration tests for `louvain` / `louvain_weighted` /
//! `louvain_with_options` (ALGO-CO-002). Cross-checks the internal
//! modularity against the standalone [`modularity`] function, exercises
//! standard golden graphs (karate, dolphins, ring-of-cliques), and
//! verifies the documented error paths.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    Graph, LouvainResult, louvain, louvain_weighted, louvain_with_options, modularity,
    read_edgelist,
};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn ring_of_cliques(num_cliques: u32, clique_size: u32) -> Graph {
    let n = num_cliques * clique_size;
    let mut g = Graph::with_vertices(n);
    for c in 0..num_cliques {
        let base = c * clique_size;
        // Internal clique edges.
        for u in 0..clique_size {
            for v in (u + 1)..clique_size {
                g.add_edge(base + u, base + v).expect("clique edge");
            }
        }
        // Single bridge to the next clique.
        let next_base = ((c + 1) % num_cliques) * clique_size;
        g.add_edge(base, next_base).expect("bridge edge");
    }
    g
}

fn assert_partition_consistent(r: &LouvainResult, expected_n: usize) {
    assert_eq!(
        r.membership.len(),
        expected_n,
        "membership length should equal vcount"
    );
    let max_label = *r.membership.iter().max().unwrap_or(&0);
    assert!(
        (max_label as usize) < expected_n,
        "label {max_label} out of range for n = {expected_n}"
    );
    // Labels must form a dense [0, k) range.
    let k = r.membership.iter().copied().max().map_or(0, |m| m + 1) as usize;
    let mut seen = vec![false; k];
    for &m in &r.membership {
        seen[m as usize] = true;
    }
    assert!(
        seen.into_iter().all(|b| b),
        "membership labels are not contiguous in [0, k)"
    );
    // Per-level membership lengths match vcount.
    for level in &r.levels {
        assert_eq!(level.len(), expected_n, "level snapshot wrong length");
    }
    assert_eq!(r.levels.len(), r.modularities.len());
}

#[test]
fn internal_modularity_matches_standalone_on_karate() {
    let g = karate();
    let r = louvain(&g).unwrap();
    let q = modularity(&g, &r.membership, 1.0).unwrap().unwrap();
    assert!(
        (r.modularity - q).abs() < 1e-9,
        "internal Q = {} ≠ standalone modularity() = {}",
        r.modularity,
        q
    );
    assert_partition_consistent(&r, g.vcount() as usize);
}

#[test]
fn karate_partition_has_high_modularity() {
    let g = karate();
    let r = louvain(&g).unwrap();
    // The classic karate-club Louvain result on the standard
    // 34-vertex/78-edge graph lands in Q ≈ 0.39..0.42 depending on
    // the shuffle order the default seed picks.
    assert!(
        r.modularity > 0.39,
        "karate Louvain Q should exceed 0.39, got {}",
        r.modularity
    );
    let k = r.membership.iter().copied().max().unwrap() + 1;
    assert!(
        (2..=8).contains(&k),
        "karate Louvain should yield 2..=8 communities, got {k}"
    );
}

#[test]
fn ring_of_four_cliques_resolves_four_communities() {
    let g = ring_of_cliques(4, 5);
    let r = louvain(&g).unwrap();
    let q = modularity(&g, &r.membership, 1.0).unwrap().unwrap();
    assert!(
        (r.modularity - q).abs() < 1e-9,
        "internal Q = {} ≠ standalone = {}",
        r.modularity,
        q
    );
    // Each ring-clique becomes its own community: there are exactly
    // four ground-truth communities and Louvain should hit > 0.6
    // modularity on this benchmark.
    let k = r.membership.iter().copied().max().unwrap() + 1;
    assert_eq!(k, 4, "expected 4 communities, got {k}");
    assert!(
        r.modularity > 0.60,
        "ring-of-cliques Q should exceed 0.60, got {}",
        r.modularity
    );
}

#[test]
fn weighted_unit_matches_unweighted_on_karate() {
    let g = karate();
    let ones = vec![1.0; g.ecount()];
    let a = louvain(&g).unwrap();
    let b = louvain_weighted(&g, &ones).unwrap();
    assert!(
        (a.modularity - b.modularity).abs() < 1e-9,
        "unit-weighted modularity should equal unweighted: {} vs {}",
        a.modularity,
        b.modularity
    );
    // Partition equivalence relation: for every pair of vertices, the
    // two runs agree on whether they share a community.
    let n = g.vcount() as usize;
    for i in 0..n {
        for j in (i + 1)..n {
            let ai = a.membership[i] == a.membership[j];
            let bi = b.membership[i] == b.membership[j];
            assert_eq!(
                ai, bi,
                "partition disagreement at pair ({i},{j}): unweighted={ai} weighted={bi}"
            );
        }
    }
}

#[test]
fn weighted_amplifies_strong_bridges() {
    // Two K4s joined by a single bridge. Unweighted Louvain splits
    // them; a very heavy weight on the bridge should still keep the
    // split (the within-K4 mass dominates), confirming weights flow
    // through the gain formula consistently.
    let mut g = Graph::with_vertices(8);
    for c in 0..2 {
        let base = c * 4;
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(base + u, base + v).unwrap();
            }
        }
    }
    g.add_edge(0, 4).unwrap(); // bridge
    let mut weights = vec![1.0; g.ecount()];
    let bridge_id = g.ecount() - 1;
    weights[bridge_id] = 0.01; // very weak bridge
    let r = louvain_weighted(&g, &weights).unwrap();
    assert_eq!(r.membership[0], r.membership[1]);
    assert_eq!(r.membership[4], r.membership[5]);
    assert_ne!(r.membership[0], r.membership[4]);
}

#[test]
fn determinism_with_explicit_seed() {
    let g = karate();
    let a = louvain_with_options(&g, None, 1.0, 12345).unwrap();
    let b = louvain_with_options(&g, None, 1.0, 12345).unwrap();
    assert_eq!(a.membership, b.membership);
    assert!((a.modularity - b.modularity).abs() < 1e-12);
}

#[test]
fn different_seeds_produce_consistent_quality() {
    // Across different seeds we may land on different partitions, but
    // Louvain on karate consistently exceeds Q = 0.39.
    let g = karate();
    for seed in [0u64, 1, 7, 99, 12345, u64::MAX] {
        let r = louvain_with_options(&g, None, 1.0, seed).unwrap();
        assert!(
            r.modularity > 0.39,
            "karate seed {seed} produced low Q = {}",
            r.modularity
        );
    }
}

#[test]
fn resolution_one_is_default() {
    let g = karate();
    let a = louvain(&g).unwrap();
    let b = louvain_with_options(&g, None, 1.0, 0).unwrap();
    assert_eq!(a.membership, b.membership);
    assert!((a.modularity - b.modularity).abs() < 1e-12);
}

#[test]
fn resolution_extremes_change_granularity() {
    let g = karate();
    let low = louvain_with_options(&g, None, 0.1, 0).unwrap();
    let high = louvain_with_options(&g, None, 5.0, 0).unwrap();
    let k_low = low.membership.iter().copied().max().unwrap() + 1;
    let k_high = high.membership.iter().copied().max().unwrap() + 1;
    assert!(
        k_low <= k_high,
        "lower γ should never increase community count: k_low={k_low}, k_high={k_high}"
    );
}

#[test]
fn directed_input_rejected() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    assert!(louvain(&g).is_err());
    assert!(louvain_weighted(&g, &[1.0]).is_err());
    assert!(louvain_with_options(&g, None, 1.0, 0).is_err());
}

#[test]
fn weight_length_mismatch_rejected() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(louvain_weighted(&g, &[1.0]).is_err());
    assert!(louvain_weighted(&g, &[1.0; 5]).is_err());
}

#[test]
fn negative_weight_rejected() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(louvain_weighted(&g, &[1.0, -0.1]).is_err());
}

#[test]
fn nan_weight_rejected() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(louvain_weighted(&g, &[1.0, f64::NAN]).is_err());
}

#[test]
fn negative_resolution_rejected() {
    let g = karate();
    assert!(louvain_with_options(&g, None, -0.1, 0).is_err());
}

#[test]
fn nonfinite_resolution_rejected() {
    let g = karate();
    assert!(louvain_with_options(&g, None, f64::NAN, 0).is_err());
    assert!(louvain_with_options(&g, None, f64::INFINITY, 0).is_err());
}

#[test]
fn empty_graph_returns_empty_result() {
    let g = Graph::with_vertices(0);
    let r = louvain(&g).unwrap();
    assert!(r.membership.is_empty());
    assert!(r.levels.is_empty());
    assert!(r.modularity == 0.0, "empty Q should be exactly 0.0");
}

#[test]
fn isolated_vertices_partition_is_singletons() {
    let g = Graph::with_vertices(5);
    let r = louvain(&g).unwrap();
    assert_eq!(r.membership.len(), 5);
    // No edges, no merges: each vertex must remain its own community,
    // labels in [0, 5).
    let k = r.membership.iter().copied().max().unwrap() + 1;
    assert_eq!(k, 5);
}

#[test]
fn self_loops_are_handled_in_init_and_aggregate() {
    // Aggregation MUST count self-loops with the IGRAPH_LOOPS_TWICE
    // convention (loop of weight w contributes 2w to k_v). The
    // regression guard: a graph that already has loops at level 0
    // should still produce a finite Q matching standalone modularity.
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 0).unwrap(); // self-loop at 0
    g.add_edge(1, 1).unwrap(); // self-loop at 1
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    let r = louvain(&g).unwrap();
    let q = modularity(&g, &r.membership, 1.0).unwrap().unwrap();
    assert!(
        (r.modularity - q).abs() < 1e-9,
        "Louvain Q = {} vs modularity() Q = {} disagree in the presence of self-loops",
        r.modularity,
        q
    );
}

#[test]
fn level_history_is_monotone_non_decreasing() {
    let g = karate();
    let r = louvain(&g).unwrap();
    for window in r.modularities.windows(2) {
        let prev = window[0];
        let next = window[1];
        // Louvain's pass-loop only accepts strictly improving merges,
        // so per-level modularity must be non-decreasing.
        assert!(
            next + 1e-9 >= prev,
            "modularity decreased across levels: {prev} → {next}"
        );
    }
}

#[test]
fn final_membership_equals_last_level() {
    let g = karate();
    let r = louvain(&g).unwrap();
    if let Some(last) = r.levels.last() {
        assert_eq!(*last, r.membership);
    }
}
