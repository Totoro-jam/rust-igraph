//! Integration tests for `spinglass` / `spinglass_weighted` /
//! `spinglass_with_options` (ALGO-CO-019). Verifies partition
//! well-formedness, golden-graph community structure, determinism,
//! error paths, update-rule variants, and temperature modes.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    Graph, SpinglassOptions, SpinglassResult, SpinglassUpdateRule, spinglass, spinglass_weighted,
    spinglass_with_options,
};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    rust_igraph::read_edgelist(File::open(path).expect("open karate fixture"))
        .expect("parse karate")
}

fn two_k4() -> Graph {
    Graph::from_edges(
        &[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 2),
            (1, 3),
            (2, 3),
            (4, 5),
            (4, 6),
            (4, 7),
            (5, 6),
            (5, 7),
            (6, 7),
            (3, 4), // bridge
        ],
        false,
        None,
    )
    .expect("two K4 cliques")
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

fn assert_partition_consistent(r: &SpinglassResult, expected_n: usize) {
    assert_eq!(r.membership.len(), expected_n);
    if expected_n == 0 {
        assert_eq!(r.nb_clusters, 0);
        return;
    }
    let k = *r.membership.iter().max().unwrap() + 1;
    assert_eq!(
        k, r.nb_clusters,
        "nb_clusters {} ≠ max(membership)+1 {}",
        r.nb_clusters, k
    );
    let mut counts = vec![0u32; k as usize];
    for &m in &r.membership {
        assert!((m as usize) < k as usize, "label out of range");
        counts[m as usize] += 1;
    }
    assert_eq!(r.csize.len(), k as usize, "csize length mismatch");
    for (i, (&expected, &actual)) in counts.iter().zip(r.csize.iter()).enumerate() {
        assert_eq!(
            expected, actual,
            "csize[{i}] = {actual} ≠ counted {expected}"
        );
    }
    let total: u32 = r.csize.iter().sum();
    assert_eq!(total as usize, expected_n);
    assert!(
        r.modularity.is_finite(),
        "modularity must be finite, got {}",
        r.modularity
    );
}

// ── Basic well-formedness ──────────────────────────────────────

#[test]
fn empty_graph() {
    let g = Graph::with_vertices(0);
    let r = spinglass(&g, None).unwrap();
    assert!(r.membership.is_empty());
    assert_eq!(r.nb_clusters, 0);
}

#[test]
fn single_vertex() {
    let g = Graph::from_edges(&[] as &[(u32, u32)], false, Some(1)).unwrap();
    let r = spinglass(&g, None).unwrap();
    assert_eq!(r.membership, vec![0]);
    assert_eq!(r.nb_clusters, 1);
}

// ── Golden-graph tests ─────────────────────────────────────────

#[test]
fn two_k4_finds_communities() {
    let g = two_k4();
    let r = spinglass(&g, None).unwrap();
    assert_partition_consistent(&r, 8);
    assert!(
        r.nb_clusters >= 2,
        "expected ≥2 clusters, got {}",
        r.nb_clusters
    );
    assert!(
        r.modularity > 0.0,
        "expected positive modularity, got {}",
        r.modularity
    );
}

#[test]
fn karate_club_reasonable_partition() {
    let g = karate();
    let r = spinglass(&g, None).unwrap();
    assert_partition_consistent(&r, 34);
    assert!(
        r.nb_clusters >= 2 && r.nb_clusters <= 10,
        "Karate club: expected 2-10 communities, got {}",
        r.nb_clusters
    );
    assert!(
        r.modularity > 0.2,
        "Karate club: modularity too low: {}",
        r.modularity
    );
}

#[test]
fn ring_of_cliques_finds_cliques() {
    let g = ring_of_cliques(4, 5);
    let r = spinglass(&g, None).unwrap();
    assert_partition_consistent(&r, 20);
    assert!(
        r.nb_clusters >= 3,
        "ring of 4 K5 cliques should find ≥3 communities, got {}",
        r.nb_clusters
    );
}

// ── Determinism ────────────────────────────────────────────────

#[test]
fn deterministic_with_same_seed() {
    let g = two_k4();
    let opts = SpinglassOptions {
        seed: 12345,
        ..SpinglassOptions::default()
    };
    let a = spinglass_with_options(&g, None, &opts).unwrap();
    let b = spinglass_with_options(&g, None, &opts).unwrap();
    assert_eq!(a.membership, b.membership);
    assert!((a.modularity - b.modularity).abs() < 1e-12);
}

#[test]
fn karate_deterministic() {
    let g = karate();
    let opts = SpinglassOptions {
        seed: 9999,
        ..SpinglassOptions::default()
    };
    let a = spinglass_with_options(&g, None, &opts).unwrap();
    let b = spinglass_with_options(&g, None, &opts).unwrap();
    assert_eq!(a.membership, b.membership);
}

// ── Weighted variant ───────────────────────────────────────────

#[test]
fn unit_weights_match_unweighted() {
    let g = two_k4();
    let a = spinglass(&g, None).unwrap();
    let ones = vec![1.0; g.ecount()];
    let b = spinglass_weighted(&g, &ones).unwrap();
    assert_eq!(a.membership, b.membership);
}

#[test]
fn strong_internal_weights() {
    let g = Graph::from_edges(
        &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)],
        false,
        None,
    )
    .unwrap();
    let weights = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.01];
    let r = spinglass_weighted(&g, &weights).unwrap();
    assert_partition_consistent(&r, 6);
    assert!(r.nb_clusters >= 2);
}

// ── Update rules ───────────────────────────────────────────────

#[test]
fn simple_update_rule_valid() {
    let g = two_k4();
    let opts = SpinglassOptions {
        update_rule: SpinglassUpdateRule::Simple,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}

#[test]
fn config_update_rule_valid() {
    let g = two_k4();
    let opts = SpinglassOptions {
        update_rule: SpinglassUpdateRule::Config,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}

#[test]
fn parallel_update_valid() {
    let g = two_k4();
    let opts = SpinglassOptions {
        parallel_update: true,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}

#[test]
fn parallel_simple_valid() {
    let g = two_k4();
    let opts = SpinglassOptions {
        parallel_update: true,
        update_rule: SpinglassUpdateRule::Simple,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}

// ── Temperature modes ──────────────────────────────────────────

#[test]
fn zero_temperature_greedy() {
    let g = two_k4();
    let opts = SpinglassOptions {
        start_temp: 0.0,
        stop_temp: 0.0,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}

#[test]
fn zero_temp_parallel() {
    let g = two_k4();
    let opts = SpinglassOptions {
        start_temp: 0.0,
        stop_temp: 0.0,
        parallel_update: true,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}

#[test]
fn custom_temperature_range() {
    let g = two_k4();
    let opts = SpinglassOptions {
        start_temp: 5.0,
        stop_temp: 0.001,
        cool_fact: 0.95,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}

// ── Gamma parameter ────────────────────────────────────────────

#[test]
fn gamma_zero_one_big_community() {
    let g = two_k4();
    let opts = SpinglassOptions {
        gamma: 0.0,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}

#[test]
fn high_gamma_more_communities() {
    let g = ring_of_cliques(4, 5);
    let low = spinglass_with_options(
        &g,
        None,
        &SpinglassOptions {
            gamma: 0.5,
            seed: 42,
            ..SpinglassOptions::default()
        },
    )
    .unwrap();
    let high = spinglass_with_options(
        &g,
        None,
        &SpinglassOptions {
            gamma: 3.0,
            seed: 42,
            ..SpinglassOptions::default()
        },
    )
    .unwrap();
    assert_partition_consistent(&low, 20);
    assert_partition_consistent(&high, 20);
    // Higher gamma tends to produce more communities
    assert!(
        high.nb_clusters >= low.nb_clusters,
        "high gamma {} communities < low gamma {} communities",
        high.nb_clusters,
        low.nb_clusters
    );
}

// ── Error paths ────────────────────────────────────────────────

#[test]
fn disconnected_graph_errors() {
    let g = Graph::from_edges(&[(0, 1), (2, 3)], false, Some(4)).unwrap();
    assert!(spinglass(&g, None).is_err());
}

#[test]
fn spins_too_low_errors() {
    let g = two_k4();
    let opts = SpinglassOptions {
        spins: 1,
        ..SpinglassOptions::default()
    };
    assert!(spinglass_with_options(&g, None, &opts).is_err());
}

#[test]
fn cool_fact_out_of_range_errors() {
    let g = two_k4();
    for &cf in &[0.0, 1.0, 1.5, -0.1] {
        let opts = SpinglassOptions {
            cool_fact: cf,
            ..SpinglassOptions::default()
        };
        assert!(
            spinglass_with_options(&g, None, &opts).is_err(),
            "cool_fact={cf} should error"
        );
    }
}

#[test]
fn negative_gamma_errors() {
    let g = two_k4();
    let opts = SpinglassOptions {
        gamma: -1.0,
        ..SpinglassOptions::default()
    };
    assert!(spinglass_with_options(&g, None, &opts).is_err());
}

#[test]
fn wrong_weight_length_errors() {
    let g = two_k4();
    assert!(spinglass(&g, Some(&[1.0])).is_err());
}

#[test]
fn negative_weight_errors() {
    let g = Graph::from_edges(&[(0, 1), (1, 2), (2, 0)], false, None).unwrap();
    assert!(spinglass(&g, Some(&[1.0, -1.0, 1.0])).is_err());
}

#[test]
fn inconsistent_temperature_errors() {
    let g = two_k4();
    // start <= stop but not both zero
    let opts = SpinglassOptions {
        start_temp: 0.5,
        stop_temp: 1.0,
        ..SpinglassOptions::default()
    };
    assert!(spinglass_with_options(&g, None, &opts).is_err());
}

// ── Spins parameter ────────────────────────────────────────────

#[test]
fn small_spins_fewer_communities() {
    let g = karate();
    let opts = SpinglassOptions {
        spins: 3,
        seed: 42,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 34);
    assert!(
        r.nb_clusters <= 3,
        "with spins=3, nb_clusters should be ≤3, got {}",
        r.nb_clusters
    );
}

#[test]
fn large_spins_valid() {
    let g = two_k4();
    let opts = SpinglassOptions {
        spins: 100,
        ..SpinglassOptions::default()
    };
    let r = spinglass_with_options(&g, None, &opts).unwrap();
    assert_partition_consistent(&r, 8);
}
