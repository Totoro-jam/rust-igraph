//! Integration tests for `label_propagation` / `label_propagation_weighted`
//! / `label_propagation_with_options` (ALGO-CO-004).
//!
//! Exercises standard golden graphs (karate, ring-of-cliques, two K4s +
//! bridge), the three algorithm variants, and the documented error
//! paths.

// `cast_possible_truncation` is allowed because test inputs are small
// constants. `float_cmp` is allowed because some assertions pin labels
// rather than floating-point values, but we keep the lint scope narrow.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    Graph, LpaOptions, LpaResult, LpaVariant, label_propagation, label_propagation_weighted,
    label_propagation_with_options, read_edgelist,
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

fn assert_partition_consistent(r: &LpaResult, expected_n: usize) {
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
}

#[test]
fn karate_partition_well_formed_default() {
    let g = karate();
    let r = label_propagation(&g).unwrap();
    assert_partition_consistent(&r, g.vcount() as usize);
    // LPA on karate typically yields 2–7 communities; we accept a
    // wide range because the algorithm is intentionally non-greedy.
    let k = r.nb_clusters;
    assert!((2..=10).contains(&k), "unexpected k = {k}");
}

#[test]
fn karate_three_variants_partition_well_formed() {
    let g = karate();
    for variant in [
        LpaVariant::Fast,
        LpaVariant::Dominance,
        LpaVariant::Retention,
    ] {
        let opts = LpaOptions {
            variant,
            seed: 0,
            ..LpaOptions::default()
        };
        let r = label_propagation_with_options(&g, None, &opts).unwrap();
        assert_partition_consistent(&r, g.vcount() as usize);
    }
}

#[test]
fn ring_of_4_cliques_detects_4_groups() {
    // 4 cliques of size 5 ⇒ a partition with k = 4 is the obvious
    // result of LPA: every vertex in the clique has the same dominant
    // label.
    let g = ring_of_cliques(4, 5);
    let opts = LpaOptions {
        seed: 0,
        ..LpaOptions::default()
    };
    let r = label_propagation_with_options(&g, None, &opts).unwrap();
    assert_eq!(r.nb_clusters, 4, "expected exactly 4 cliques");
    for c in 0..4 {
        let base = (c * 5) as usize;
        let label = r.membership[base];
        for offset in 1..5 {
            assert_eq!(
                r.membership[base + offset],
                label,
                "vertex {} not in same cluster as {}",
                base + offset,
                base
            );
        }
    }
}

#[test]
fn two_k4s_with_thin_bridge_split() {
    // K4 + K4 joined by a single bridge edge; the bridge should be cut.
    let mut g = Graph::with_vertices(8);
    for u in 0..4 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).unwrap();
        }
    }
    for u in 4..8 {
        for v in (u + 1)..8 {
            g.add_edge(u, v).unwrap();
        }
    }
    g.add_edge(3, 4).unwrap();
    let r = label_propagation(&g).unwrap();
    for u in 0..4 {
        assert_eq!(r.membership[u], r.membership[0]);
    }
    for u in 4..8 {
        assert_eq!(r.membership[u], r.membership[4]);
    }
    assert_ne!(r.membership[0], r.membership[4]);
}

#[test]
fn unit_weights_match_unweighted() {
    let g = karate();
    let opts = LpaOptions {
        seed: 42,
        ..LpaOptions::default()
    };
    let a = label_propagation_with_options(&g, None, &opts).unwrap();
    let ones = vec![1.0; g.ecount()];
    let b = label_propagation_with_options(&g, Some(&ones), &opts).unwrap();
    assert_eq!(
        a.membership, b.membership,
        "unit-weighted partition should match unweighted"
    );
}

#[test]
fn weighted_bridge_collapse() {
    // K3+K3 with a very thin bridge (w = 0.001) vs heavy intra-clique
    // weights — LPA should keep the bridge cut.
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let weights = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.001];
    let r = label_propagation_weighted(&g, &weights).unwrap();
    assert_ne!(r.membership[0], r.membership[3]);
}

#[test]
fn determinism_under_seed_all_variants() {
    let g = karate();
    for variant in [
        LpaVariant::Fast,
        LpaVariant::Dominance,
        LpaVariant::Retention,
    ] {
        let opts = LpaOptions {
            variant,
            seed: 12345,
            ..LpaOptions::default()
        };
        let a = label_propagation_with_options(&g, None, &opts).unwrap();
        let b = label_propagation_with_options(&g, None, &opts).unwrap();
        assert_eq!(a.membership, b.membership);
        assert_eq!(a.nb_clusters, b.nb_clusters);
    }
}

#[test]
fn fixed_vertices_preserve_co_membership() {
    // Fix vertex 0 with label 0 and vertex 1 with label 1 in a K3 —
    // they must stay in separate communities.
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 2).unwrap();
    g.add_edge(1, 2).unwrap();
    let opts = LpaOptions {
        initial: Some(vec![0, 1, 2]),
        fixed: Some(vec![true, true, false]),
        ..LpaOptions::default()
    };
    let r = label_propagation_with_options(&g, None, &opts).unwrap();
    assert_ne!(r.membership[0], r.membership[1]);
}

#[test]
fn unlabelled_isolates_get_separate_labels() {
    // 5 isolated vertices, all unlabelled. Each must become its own
    // singleton community.
    let g = Graph::with_vertices(5);
    let opts = LpaOptions {
        initial: Some(vec![-1; 5]),
        ..LpaOptions::default()
    };
    let r = label_propagation_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 5);
    assert_eq!(r.nb_clusters, 5);
}

#[test]
fn ring_of_cliques_8x5() {
    let g = ring_of_cliques(8, 5);
    let opts = LpaOptions {
        variant: LpaVariant::Fast,
        seed: 0,
        ..LpaOptions::default()
    };
    let r = label_propagation_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, g.vcount() as usize);
    // For a ring-of-cliques the natural partition has 8 communities;
    // LPA is greedy and may merge a couple under some seeds. Accept a
    // window of 5..=8.
    let k = r.nb_clusters;
    assert!((5..=8).contains(&k), "unexpected k = {k}");
}

#[test]
fn directed_input_rejected() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    assert!(label_propagation(&g).is_err());
    assert!(label_propagation_weighted(&g, &[1.0]).is_err());
    assert!(label_propagation_with_options(&g, None, &LpaOptions::default()).is_err());
}

#[test]
fn invalid_weights_rejected() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(label_propagation_weighted(&g, &[1.0]).is_err());
    assert!(label_propagation_weighted(&g, &[1.0, f64::NAN]).is_err());
    assert!(label_propagation_weighted(&g, &[1.0, -0.1]).is_err());
}

#[test]
fn invalid_initial_rejected() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    // wrong length
    let opts = LpaOptions {
        initial: Some(vec![0, 1]),
        ..LpaOptions::default()
    };
    assert!(label_propagation_with_options(&g, None, &opts).is_err());
    // out-of-range label
    let opts = LpaOptions {
        initial: Some(vec![0, 1, 99]),
        ..LpaOptions::default()
    };
    assert!(label_propagation_with_options(&g, None, &opts).is_err());
}

#[test]
fn invalid_fixed_rejected() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    let opts = LpaOptions {
        initial: Some(vec![0, 1, 2]),
        fixed: Some(vec![false, false]),
        ..LpaOptions::default()
    };
    assert!(label_propagation_with_options(&g, None, &opts).is_err());
}

#[test]
fn empty_graph_returns_empty() {
    let g = Graph::with_vertices(0);
    let r = label_propagation(&g).unwrap();
    assert_eq!(r.membership.len(), 0);
    assert_eq!(r.nb_clusters, 0);
}
