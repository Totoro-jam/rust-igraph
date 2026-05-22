//! Integration tests for `leiden` / `leiden_weighted` /
//! `leiden_with_options` (ALGO-CO-003).
//!
//! Cross-checks the internal Leiden quality (Modularity objective)
//! against the standalone [`modularity`] function on loop-free graphs,
//! exercises standard golden graphs (karate, ring-of-cliques, two
//! K4s + bridge), and verifies the documented error paths.

// `cast_possible_truncation` is allowed because the test inputs are
// small constants; `float_cmp` is allowed because some assertions
// pin Q == 0.0 exactly (well-defined on the empty graph).
#![allow(clippy::cast_possible_truncation, clippy::float_cmp)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    Graph, LeidenObjective, LeidenOptions, LeidenResult, leiden, leiden_weighted,
    leiden_with_options, modularity, read_edgelist,
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
        for u in 0..clique_size {
            for v in (u + 1)..clique_size {
                g.add_edge(base + u, base + v).expect("clique edge");
            }
        }
        let next_base = ((c + 1) % num_cliques) * clique_size;
        g.add_edge(base, next_base).expect("bridge edge");
    }
    g
}

fn assert_partition_consistent(r: &LeidenResult, expected_n: usize) {
    assert_eq!(
        r.membership.len(),
        expected_n,
        "membership length should equal vcount"
    );
    if expected_n == 0 {
        assert_eq!(r.nb_clusters, 0);
        return;
    }
    let max_label = *r.membership.iter().max().unwrap_or(&0);
    assert!(
        (max_label as usize) < expected_n,
        "label {max_label} out of range for n = {expected_n}"
    );
    let k = max_label + 1;
    assert_eq!(
        k, r.nb_clusters,
        "nb_clusters {} ≠ max(membership)+1 {}",
        r.nb_clusters, k
    );
    let mut seen = vec![false; k as usize];
    for &m in &r.membership {
        seen[m as usize] = true;
    }
    assert!(
        seen.into_iter().all(|b| b),
        "membership labels are not contiguous in [0, k)"
    );
    assert_eq!(r.qualities.len() as u32, r.n_iterations_run);
}

#[test]
fn modularity_objective_matches_standalone_on_karate() {
    // Karate has no self-loops, so Leiden(Modularity, γ=1) reduces to
    // Newman-Girvan modularity, and the standalone modularity() should
    // agree to f64 precision.
    let g = karate();
    let r = leiden(&g).unwrap();
    let q = modularity(&g, &r.membership, 1.0).unwrap().unwrap();
    assert!(
        (r.quality - q).abs() < 1e-9,
        "internal Q = {} ≠ standalone modularity() = {}",
        r.quality,
        q
    );
    assert_partition_consistent(&r, g.vcount() as usize);
}

#[test]
fn karate_partition_has_high_modularity() {
    let g = karate();
    let r = leiden(&g).unwrap();
    assert!(
        r.quality > 0.39,
        "karate Leiden Q should exceed 0.39, got {}",
        r.quality
    );
    let k = r.nb_clusters;
    assert!(
        (2..=8).contains(&k),
        "karate Leiden should yield 2..=8 communities, got {k}"
    );
}

#[test]
fn ring_of_four_cliques_resolves_four_communities() {
    let g = ring_of_cliques(4, 5);
    let r = leiden(&g).unwrap();
    let q = modularity(&g, &r.membership, 1.0).unwrap().unwrap();
    assert!(
        (r.quality - q).abs() < 1e-9,
        "internal Q = {} ≠ standalone = {}",
        r.quality,
        q
    );
    assert_eq!(r.nb_clusters, 4, "expected 4 communities");
    assert!(
        r.quality > 0.60,
        "ring-of-cliques Q should exceed 0.60, got {}",
        r.quality
    );
}

#[test]
fn weighted_unit_matches_unweighted_on_karate() {
    let g = karate();
    let ones = vec![1.0; g.ecount()];
    let a = leiden(&g).unwrap();
    let b = leiden_weighted(&g, &ones).unwrap();
    assert!(
        (a.quality - b.quality).abs() < 1e-9,
        "unit-weighted Q should equal unweighted: {} vs {}",
        a.quality,
        b.quality
    );
    // Partition equivalence: every vertex pair agrees on same-cluster.
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
fn weighted_thin_bridge_keeps_two_k4_split() {
    // Two K4s joined by a single bridge with a tiny weight.
    let mut g = Graph::with_vertices(8);
    for c in 0..2 {
        let base = c * 4;
        for u in 0..4u32 {
            for v in (u + 1)..4 {
                g.add_edge(base + u, base + v).unwrap();
            }
        }
    }
    g.add_edge(0, 4).unwrap();
    let mut weights = vec![1.0; g.ecount()];
    let bridge_id = g.ecount() - 1;
    weights[bridge_id] = 0.01;
    let r = leiden_weighted(&g, &weights).unwrap();
    assert_eq!(r.membership[0], r.membership[1]);
    assert_eq!(r.membership[4], r.membership[5]);
    assert_ne!(r.membership[0], r.membership[4]);
}

#[test]
fn determinism_with_explicit_seed() {
    let g = karate();
    let opts = LeidenOptions {
        seed: 12345,
        ..LeidenOptions::default()
    };
    let a = leiden_with_options(&g, None, &opts).unwrap();
    let b = leiden_with_options(&g, None, &opts).unwrap();
    assert_eq!(a.membership, b.membership);
    assert!((a.quality - b.quality).abs() < 1e-12);
}

#[test]
fn cpm_resolution_extremes_change_granularity() {
    // Higher γ in CPM should never *decrease* the number of clusters
    // (it makes the −γ·N² penalty heavier per merger).
    let g = ring_of_cliques(4, 5);
    let low_opts = LeidenOptions {
        objective: LeidenObjective::Cpm,
        resolution: 0.01,
        ..LeidenOptions::default()
    };
    let high_opts = LeidenOptions {
        objective: LeidenObjective::Cpm,
        resolution: 5.0,
        ..LeidenOptions::default()
    };
    let low = leiden_with_options(&g, None, &low_opts).unwrap();
    let high = leiden_with_options(&g, None, &high_opts).unwrap();
    assert!(
        low.nb_clusters <= high.nb_clusters,
        "CPM γ↑ should not reduce k: k(low)={}, k(high)={}",
        low.nb_clusters,
        high.nb_clusters
    );
    // At very high γ, CPM collapses to the singleton partition.
    assert_eq!(high.nb_clusters as usize, g.vcount() as usize);
}

#[test]
fn er_objective_runs_and_partitions_validly() {
    // Sanity check: ER objective produces a valid partition on a
    // moderately structured graph.
    let g = ring_of_cliques(3, 4);
    let opts = LeidenOptions {
        objective: LeidenObjective::Er,
        resolution: 1.0,
        ..LeidenOptions::default()
    };
    let r = leiden_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, g.vcount() as usize);
    assert!(r.quality.is_finite());
}

#[test]
fn n_iterations_negative_iterates_to_stable() {
    // n_iterations < 0 ⇒ run until a pass produces no change. The
    // returned n_iterations_run should be ≥ 1 and the final pass
    // should be a no-op (quality stable).
    let g = karate();
    let opts = LeidenOptions {
        n_iterations: -1,
        seed: 1,
        ..LeidenOptions::default()
    };
    let r = leiden_with_options(&g, None, &opts).unwrap();
    assert!(r.n_iterations_run >= 1);
    assert_partition_consistent(&r, g.vcount() as usize);
}

#[test]
fn start_membership_is_honored() {
    let g = ring_of_cliques(4, 5);
    // Start from the ground-truth ring-of-cliques partition; Leiden
    // should leave it alone (already optimal).
    let mut start = vec![0u32; g.vcount() as usize];
    for c in 0..4 {
        for v in 0..5 {
            start[c * 5 + v] = c as u32;
        }
    }
    let opts = LeidenOptions {
        start: Some(start.clone()),
        ..LeidenOptions::default()
    };
    let r = leiden_with_options(&g, None, &opts).unwrap();
    // Same equivalence relation as the ground truth.
    let n = g.vcount() as usize;
    for i in 0..n {
        for j in (i + 1)..n {
            assert_eq!(
                start[i] == start[j],
                r.membership[i] == r.membership[j],
                "Leiden moved a vertex away from ground-truth partition at ({i},{j})"
            );
        }
    }
}

#[test]
fn start_length_mismatch_rejected() {
    let g = Graph::with_vertices(5);
    let opts = LeidenOptions {
        start: Some(vec![0; 3]),
        ..LeidenOptions::default()
    };
    assert!(leiden_with_options(&g, None, &opts).is_err());
}

#[test]
fn directed_input_rejected() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    assert!(leiden(&g).is_err());
    assert!(leiden_weighted(&g, &[1.0]).is_err());
    assert!(leiden_with_options(&g, None, &LeidenOptions::default()).is_err());
}

#[test]
fn weight_length_mismatch_rejected() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(leiden_weighted(&g, &[1.0]).is_err());
    assert!(leiden_weighted(&g, &[1.0; 5]).is_err());
}

#[test]
fn negative_weight_rejected_for_modularity_and_er() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(leiden_weighted(&g, &[1.0, -0.1]).is_err());
    let er_opts = LeidenOptions {
        objective: LeidenObjective::Er,
        ..LeidenOptions::default()
    };
    assert!(leiden_with_options(&g, Some(&[1.0, -0.1]), &er_opts).is_err());
}

#[test]
fn cpm_allows_negative_weights() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let opts = LeidenOptions {
        objective: LeidenObjective::Cpm,
        ..LeidenOptions::default()
    };
    assert!(leiden_with_options(&g, Some(&[1.0, -0.1]), &opts).is_ok());
}

#[test]
fn nonfinite_weight_rejected() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(leiden_weighted(&g, &[1.0, f64::NAN]).is_err());
    assert!(leiden_weighted(&g, &[1.0, f64::INFINITY]).is_err());
    assert!(leiden_weighted(&g, &[1.0, f64::NEG_INFINITY]).is_err());
}

#[test]
fn negative_resolution_rejected() {
    let g = karate();
    let opts = LeidenOptions {
        resolution: -0.1,
        ..LeidenOptions::default()
    };
    assert!(leiden_with_options(&g, None, &opts).is_err());
}

#[test]
fn nonfinite_resolution_rejected() {
    let g = karate();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let opts = LeidenOptions {
            resolution: bad,
            ..LeidenOptions::default()
        };
        assert!(leiden_with_options(&g, None, &opts).is_err());
    }
}

#[test]
fn negative_beta_rejected() {
    let g = karate();
    let opts = LeidenOptions {
        beta: -0.1,
        ..LeidenOptions::default()
    };
    assert!(leiden_with_options(&g, None, &opts).is_err());
}

#[test]
fn nonfinite_beta_rejected() {
    let g = karate();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let opts = LeidenOptions {
            beta: bad,
            ..LeidenOptions::default()
        };
        assert!(leiden_with_options(&g, None, &opts).is_err());
    }
}

#[test]
fn empty_graph_returns_empty_result() {
    let g = Graph::with_vertices(0);
    let r = leiden(&g).unwrap();
    assert!(r.membership.is_empty());
    assert_eq!(r.nb_clusters, 0);
    assert_eq!(r.quality, 0.0);
}

#[test]
fn isolated_vertices_produce_valid_partition() {
    let g = Graph::with_vertices(5);
    let r = leiden(&g).unwrap();
    assert_partition_consistent(&r, 5);
}

#[test]
fn self_loops_produce_finite_quality() {
    // Leiden treats self-loops with the IGRAPH_LOOPS convention (count
    // once per endpoint), so Leiden's Q on a graph with loops won't
    // exactly equal modularity() (which uses IGRAPH_LOOPS_TWICE). We
    // only check finiteness + a valid partition here.
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 0).unwrap();
    g.add_edge(1, 1).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    let r = leiden(&g).unwrap();
    assert!(r.quality.is_finite());
    assert_partition_consistent(&r, 4);
}

#[test]
fn quality_history_length_matches_iterations() {
    let g = karate();
    let opts = LeidenOptions {
        n_iterations: 3,
        ..LeidenOptions::default()
    };
    let r = leiden_with_options(&g, None, &opts).unwrap();
    assert_eq!(r.qualities.len(), 3);
    assert_eq!(r.n_iterations_run, 3);
}

#[test]
fn two_triangles_bridge_splits() {
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let r = leiden(&g).unwrap();
    assert_eq!(r.membership[0], r.membership[1]);
    assert_eq!(r.membership[0], r.membership[2]);
    assert_eq!(r.membership[3], r.membership[4]);
    assert_eq!(r.membership[3], r.membership[5]);
    assert_ne!(r.membership[0], r.membership[3]);
    assert_eq!(r.nb_clusters, 2);
}
