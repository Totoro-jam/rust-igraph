//! Integration tests for `infomap` / `infomap_weighted` /
//! `infomap_with_options` (ALGO-CO-018). Verifies partition well-formedness,
//! golden-graph community structure, determinism, error paths, and
//! agreement between weighted/unweighted variants.

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{Graph, InfomapResult, infomap, infomap_weighted, infomap_with_options};

fn karate() -> Graph {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/karate.edges");
    rust_igraph::read_edgelist(File::open(path).expect("open karate fixture"))
        .expect("parse karate")
}

fn two_triangles() -> Graph {
    Graph::from_edges(
        &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
        false,
        None,
    )
    .expect("two triangles")
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

fn assert_partition_consistent(r: &InfomapResult, expected_n: usize) {
    assert_eq!(r.membership.len(), expected_n);
    if expected_n == 0 {
        return;
    }
    let k = *r.membership.iter().max().unwrap() + 1;
    let mut seen = vec![false; k as usize];
    for &m in &r.membership {
        assert!((m as usize) < k as usize, "label out of range");
        seen[m as usize] = true;
    }
    assert!(
        seen.iter().all(|&b| b),
        "membership labels must be contiguous in [0, k)"
    );
    assert!(r.codelength.is_finite(), "codelength must be finite");
}

// ── Basic well-formedness ──────────────────────────────────────

#[test]
fn empty_graph() {
    let g = Graph::with_vertices(0);
    let r = infomap(&g).unwrap();
    assert!(r.membership.is_empty());
}

#[test]
fn single_vertex_no_edges() {
    let g = Graph::with_vertices(1);
    let r = infomap(&g).unwrap();
    assert_eq!(r.membership, vec![0]);
}

#[test]
fn single_edge() {
    let g = Graph::from_edges(&[(0u32, 1u32)], false, None).unwrap();
    let r = infomap(&g).unwrap();
    assert_partition_consistent(&r, 2);
}

// ── Golden-graph tests ─────────────────────────────────────────

#[test]
fn two_triangles_finds_two_communities() {
    let g = two_triangles();
    let r = infomap(&g).unwrap();
    assert_partition_consistent(&r, 6);
    // Vertices within the same triangle should be in the same community
    assert_eq!(r.membership[0], r.membership[1]);
    assert_eq!(r.membership[0], r.membership[2]);
    assert_eq!(r.membership[3], r.membership[4]);
    assert_eq!(r.membership[3], r.membership[5]);
    // The two triangles should be in different communities
    assert_ne!(r.membership[0], r.membership[3]);
}

#[test]
fn karate_club_reasonable_partition() {
    let g = karate();
    let r = infomap(&g).unwrap();
    assert_partition_consistent(&r, 34);
    let k = *r.membership.iter().max().unwrap() + 1;
    assert!(
        (2..=10).contains(&k),
        "Karate club should yield 2-10 communities, got {k}"
    );
}

#[test]
fn ring_of_cliques_finds_cliques() {
    let g = ring_of_cliques(4, 5);
    let r = infomap(&g).unwrap();
    assert_partition_consistent(&r, 20);
    let k = *r.membership.iter().max().unwrap() + 1;
    assert!(
        k >= 3,
        "ring of 4 K5 cliques should find ≥3 communities, got {k}"
    );
}

// ── Determinism ────────────────────────────────────────────────

#[test]
fn deterministic_with_same_seed() {
    let g = karate();
    let a = infomap_with_options(&g, None, None, 3, 42).unwrap();
    let b = infomap_with_options(&g, None, None, 3, 42).unwrap();
    assert_eq!(a.membership, b.membership);
    assert!((a.codelength - b.codelength).abs() < 1e-12);
}

#[test]
fn different_seeds_may_differ() {
    let g = karate();
    let _a = infomap_with_options(&g, None, None, 1, 1).unwrap();
    let _b = infomap_with_options(&g, None, None, 1, 999).unwrap();
    // Not asserting they differ — just that both run without error
}

// ── Weighted variant ───────────────────────────────────────────

#[test]
fn unit_weights_match_unweighted() {
    let g = karate();
    let a = infomap(&g).unwrap();
    let ones = vec![1.0; g.ecount()];
    let b = infomap_weighted(&g, &ones).unwrap();
    assert_eq!(a.membership, b.membership);
}

#[test]
fn strong_internal_weights_separate_communities() {
    let g = two_triangles();
    let weights = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.01];
    let r = infomap_weighted(&g, &weights).unwrap();
    assert_partition_consistent(&r, 6);
    assert_eq!(r.membership[0], r.membership[1]);
    assert_ne!(r.membership[0], r.membership[3]);
}

// ── Error paths ────────────────────────────────────────────────

#[test]
fn wrong_weight_length_errors() {
    let g = two_triangles();
    let result = infomap_weighted(&g, &[1.0]);
    assert!(result.is_err());
}

#[test]
fn negative_weight_errors() {
    let g = two_triangles();
    let weights = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
    let result = infomap_weighted(&g, &weights);
    assert!(result.is_err());
}

#[test]
fn zero_trials_errors() {
    let g = two_triangles();
    let result = infomap_with_options(&g, None, None, 0, 0);
    assert!(result.is_err());
}

// ── Multi-trial improvement ────────────────────────────────────

#[test]
fn multi_trial_no_worse_than_single() {
    let g = ring_of_cliques(4, 5);
    let r1 = infomap_with_options(&g, None, None, 1, 42).unwrap();
    let r5 = infomap_with_options(&g, None, None, 5, 42).unwrap();
    assert!(
        r5.codelength <= r1.codelength + 1e-9,
        "5-trial codelength {} > 1-trial {}",
        r5.codelength,
        r1.codelength
    );
}

// ── Vertex weights ─────────────────────────────────────────────

#[test]
fn custom_vertex_weights() {
    let g = two_triangles();
    let vw: Vec<f64> = (0..6).map(|i| f64::from(i) + 1.0).collect();
    let r = infomap_with_options(&g, None, Some(&vw), 1, 42).unwrap();
    assert_partition_consistent(&r, 6);
}

#[test]
fn wrong_vertex_weight_length_errors() {
    let g = two_triangles();
    let result = infomap_with_options(&g, None, Some(&[1.0]), 1, 0);
    assert!(result.is_err());
}
