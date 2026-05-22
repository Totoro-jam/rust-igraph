//! Integration tests for `fast_greedy_modularity` (ALGO-CO-007).
//!
//! All expected modularity values come from the upstream igraph C test
//! `references/igraph/tests/unit/igraph_community_fastgreedy.c` and its
//! `.out` artefact, so the Rust port is pinned to the same numeric
//! contract as the C reference.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::many_single_char_names
)]

use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    FastGreedyResult, Graph, fast_greedy_modularity, fast_greedy_modularity_weighted, read_edgelist,
};

const TOL: f64 = 1e-5;

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    read_edgelist(File::open(path).expect("open karate fixture")).expect("parse karate")
}

fn best_q(r: &FastGreedyResult) -> f64 {
    r.modularity
        .iter()
        .copied()
        .filter(|q| !q.is_nan())
        .fold(f64::NEG_INFINITY, f64::max)
}

fn assert_well_formed(r: &FastGreedyResult, n: u32) {
    assert_eq!(r.membership.len() as u32, n, "membership length");
    assert!(r.modularity.len() == r.merges.len() + 1, "Q traj length");
    for &lbl in &r.membership {
        assert!(lbl < r.nb_clusters, "dense label out of range");
    }
    let distinct: HashSet<u32> = r.membership.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        r.nb_clusters as usize,
        "nb_clusters matches"
    );
}

// ---------- upstream igraph C `igraph_community_fastgreedy.c` ----------

/// Two K5 cliques joined by a single bridge.
#[test]
fn two_k5_bridge_matches_upstream() {
    let mut g = Graph::with_vertices(10);
    for u in 0..5u32 {
        for v in (u + 1)..5 {
            g.add_edge(u, v).expect("clique edge");
        }
    }
    for u in 5..10u32 {
        for v in (u + 1)..10 {
            g.add_edge(u, v).expect("clique edge");
        }
    }
    g.add_edge(0, 5).expect("bridge");

    let r = fast_greedy_modularity(&g).unwrap();
    assert_well_formed(&r, 10);
    let q = best_q(&r);
    assert!(
        (q - 0.452_381).abs() < TOL,
        "two-K5-bridge: best Q = {q}, expected 0.452381"
    );
    assert_eq!(r.nb_clusters, 2);
    for v in 1..5 {
        assert_eq!(r.membership[v], r.membership[0]);
    }
    for v in 6..10 {
        assert_eq!(r.membership[v], r.membership[5]);
    }
    assert_ne!(r.membership[0], r.membership[5]);
}

/// Same graph as above, weights = 2.0 — should be invariant under
/// uniform positive rescaling.
#[test]
fn two_k5_bridge_uniform_weights_match_unweighted() {
    let mut g = Graph::with_vertices(10);
    for u in 0..5u32 {
        for v in (u + 1)..5 {
            g.add_edge(u, v).unwrap();
        }
    }
    for u in 5..10u32 {
        for v in (u + 1)..10 {
            g.add_edge(u, v).unwrap();
        }
    }
    g.add_edge(0, 5).unwrap();
    let weights = vec![2.0_f64; g.ecount()];
    let r = fast_greedy_modularity_weighted(&g, &weights).unwrap();
    let q = best_q(&r);
    assert!(
        (q - 0.452_381).abs() < TOL,
        "two-K5-bridge weighted: best Q = {q}, expected 0.452381"
    );
    assert_eq!(r.nb_clusters, 2);
}

/// K4 + K4 + isolated vertex: Q = 0.5 (upstream).
#[test]
fn two_k4_with_isolate_matches_upstream() {
    let mut g = Graph::with_vertices(9);
    for u in 0..4u32 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).unwrap();
        }
    }
    for u in 4..8u32 {
        for v in (u + 1)..8 {
            g.add_edge(u, v).unwrap();
        }
    }
    // vertex 8 is isolated
    let r = fast_greedy_modularity(&g).unwrap();
    assert_well_formed(&r, 9);
    let q = best_q(&r);
    assert!(
        (q - 0.5).abs() < TOL,
        "K4+K4+iso: best Q = {q}, expected 0.5"
    );
    // Each K4 forms its own community; the isolate sits in its own.
    assert_eq!(r.nb_clusters, 3);
    for v in 1..4 {
        assert_eq!(r.membership[v], r.membership[0]);
    }
    for v in 5..8 {
        assert_eq!(r.membership[v], r.membership[4]);
    }
    assert_ne!(r.membership[0], r.membership[4]);
    assert_ne!(r.membership[0], r.membership[8]);
    assert_ne!(r.membership[4], r.membership[8]);
}

/// Two disjoint 10-cycles: Q = 0.54 (upstream).
#[test]
fn two_disjoint_10_rings_matches_upstream() {
    let mut g = Graph::with_vertices(20);
    for u in 0..10u32 {
        let v = (u + 1) % 10;
        g.add_edge(u, v).unwrap();
    }
    for u in 10..20u32 {
        let next = 10 + ((u - 10) + 1) % 10;
        g.add_edge(u, next).unwrap();
    }
    let r = fast_greedy_modularity(&g).unwrap();
    assert_well_formed(&r, 20);
    let q = best_q(&r);
    assert!(
        (q - 0.54).abs() < TOL,
        "two-disjoint-10-rings: best Q = {q}, expected 0.54"
    );
    // Each ring partitions into 2 halves => 4 communities.
    assert_eq!(r.nb_clusters, 4);
}

/// Karate club (34v 78e). Upstream `.out`: Q = 0.380671 with 3 communities.
#[test]
fn karate_matches_upstream() {
    let g = karate();
    let r = fast_greedy_modularity(&g).unwrap();
    assert_well_formed(&r, g.vcount());
    let q = best_q(&r);
    assert!(
        (q - 0.380_671).abs() < TOL,
        "karate: best Q = {q}, expected ≈ 0.380671"
    );
    assert_eq!(r.nb_clusters, 3);
}

/// 6-vertex graph from upstream: edges 0-1, 1-2, 2-3, 2-4, 2-5, 3-4, 3-5, 4-5.
/// Unweighted: Q = 0.179688.
#[test]
fn small_6v8e_unweighted_matches_upstream() {
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[
        (0u32, 1u32),
        (1, 2),
        (2, 3),
        (2, 4),
        (2, 5),
        (3, 4),
        (3, 5),
        (4, 5),
    ] {
        g.add_edge(u, v).unwrap();
    }
    let r = fast_greedy_modularity(&g).unwrap();
    assert_well_formed(&r, 6);
    let q = best_q(&r);
    assert!(
        (q - 0.179_688).abs() < 1e-4,
        "6v8e unweighted: best Q = {q}, expected 0.179688"
    );
    // Upstream membership pattern: vertices 0,1 vs the rest.
    assert_eq!(r.nb_clusters, 2);
    assert_eq!(r.membership[0], r.membership[1]);
    for v in 2..6 {
        assert_eq!(r.membership[v], r.membership[2]);
    }
    assert_ne!(r.membership[0], r.membership[2]);
}

/// Edgeless 10v: Q = NaN, all singletons.
#[test]
fn edgeless_yields_nan_singletons() {
    let g = Graph::with_vertices(10);
    let r = fast_greedy_modularity(&g).unwrap();
    assert_well_formed(&r, 10);
    assert_eq!(r.nb_clusters, 10);
    assert!(r.merges.is_empty());
    assert_eq!(r.modularity.len(), 1);
    assert!(r.modularity[0].is_nan());
}

/// Two isolated vertices, each with a self-loop. Upstream Q = 0.5.
#[test]
fn two_vertices_with_self_loops_matches_upstream() {
    let mut g = Graph::with_vertices(2);
    g.add_edge(0, 0).unwrap();
    g.add_edge(1, 1).unwrap();
    let r = fast_greedy_modularity(&g).unwrap();
    assert_well_formed(&r, 2);
    let q = best_q(&r);
    assert!(
        (q - 0.5).abs() < TOL,
        "2v+2loops: best Q = {q}, expected 0.5"
    );
    assert_eq!(r.nb_clusters, 2);
    assert_ne!(r.membership[0], r.membership[1]);
}

// ---------- error paths ----------

#[test]
fn rejects_directed_graph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    assert!(fast_greedy_modularity(&g).is_err());
}

#[test]
fn rejects_multi_edges() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(fast_greedy_modularity(&g).is_err());
}

#[test]
fn rejects_negative_weight() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(fast_greedy_modularity_weighted(&g, &[1.0, -0.5]).is_err());
}

#[test]
fn rejects_nan_weight() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(fast_greedy_modularity_weighted(&g, &[1.0, f64::NAN]).is_err());
}

#[test]
fn rejects_weight_length_mismatch() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(fast_greedy_modularity_weighted(&g, &[1.0]).is_err());
}

#[test]
fn weighted_and_unweighted_agree_on_unit_weights() {
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0u32, 1u32), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (0, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let r1 = fast_greedy_modularity(&g).unwrap();
    let ws = vec![1.0_f64; g.ecount()];
    let r2 = fast_greedy_modularity_weighted(&g, &ws).unwrap();
    assert_eq!(r1.merges, r2.merges);
    assert_eq!(r1.membership, r2.membership);
    assert_eq!(r1.nb_clusters, r2.nb_clusters);
    for (a, b) in r1.modularity.iter().zip(r2.modularity.iter()) {
        assert!(
            (a - b).abs() < 1e-12 || (a.is_nan() && b.is_nan()),
            "Q mismatch a={a} b={b}"
        );
    }
}
