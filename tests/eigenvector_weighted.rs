//! Integration tests for `eigenvector_centrality_weighted` (ALGO-PR-012b).
//!
//! Covers length validation, parity with the unweighted path under
//! unit weights, scaling invariance, all-zero / empty sentinels, and
//! the upstream golden weighted star.

use rust_igraph::{Graph, eigenvector_centrality, eigenvector_centrality_weighted};

fn close(actual: &[f64], expected: &[f64], tol: f64) {
    assert_eq!(actual.len(), expected.len(), "length mismatch");
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            (a - e).abs() < tol,
            "index {i}: actual={a} expected={e} tol={tol}"
        );
    }
}

#[test]
fn weighted_unit_matches_unweighted_triangle() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    let unw = eigenvector_centrality(&g).unwrap();
    let w = eigenvector_centrality_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
    close(&w.vector, &unw, 1e-9);
    assert!((w.eigenvalue - 2.0).abs() < 1e-9);
}

#[test]
fn weighted_unit_matches_unweighted_k5() {
    let mut g = Graph::with_vertices(5);
    for u in 0..5u32 {
        for v in (u + 1)..5 {
            g.add_edge(u, v).unwrap();
        }
    }
    let unw = eigenvector_centrality(&g).unwrap();
    let w = eigenvector_centrality_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
    close(&w.vector, &unw, 1e-9);
    assert!((w.eigenvalue - 4.0).abs() < 1e-9);
}

#[test]
fn weighted_scaling_preserves_vector_scales_eigenvalue() {
    // Scaling every weight by c scales every entry of W by c, so the
    // dominant eigenvector is unchanged but the eigenvalue scales.
    let mut g = Graph::with_vertices(4);
    for u in 0..4u32 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).unwrap();
        }
    }
    let s1 = eigenvector_centrality_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
    let s2 = eigenvector_centrality_weighted(&g, &vec![3.0; g.ecount()]).unwrap();
    close(&s1.vector, &s2.vector, 1e-9);
    assert!(
        (s2.eigenvalue / s1.eigenvalue - 3.0).abs() < 1e-6,
        "expected 3x eigenvalue, got {} vs {}",
        s2.eigenvalue,
        s1.eigenvalue,
    );
}

#[test]
fn weighted_star_unit_matches_upstream() {
    // Upstream golden: K_{1,4} star, unit weights → λ=2, vec=[1, 0.5, 0.5, 0.5, 0.5].
    let mut g = Graph::with_vertices(5);
    for v in 1..5 {
        g.add_edge(0, v).unwrap();
    }
    let s = eigenvector_centrality_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
    close(&s.vector, &[1.0, 0.5, 0.5, 0.5, 0.5], 1e-9);
    assert!((s.eigenvalue - 2.0).abs() < 1e-9);
}

#[test]
fn weighted_length_mismatch_errors() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let err = eigenvector_centrality_weighted(&g, &[1.0]);
    assert!(err.is_err(), "expected length-mismatch error");
}

#[test]
fn weighted_empty_returns_ones() {
    let g = Graph::with_vertices(4);
    let s = eigenvector_centrality_weighted(&g, &[]).unwrap();
    close(&s.vector, &[1.0; 4], 1e-15);
    assert!(s.eigenvalue.abs() < f64::EPSILON);
}

#[test]
fn weighted_all_zero_returns_ones() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let s = eigenvector_centrality_weighted(&g, &[0.0, 0.0]).unwrap();
    close(&s.vector, &[1.0; 3], 1e-15);
    assert!(s.eigenvalue.abs() < f64::EPSILON);
}

#[test]
fn weighted_directed_input_returns_error() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    // The undirected-only signature rejects directed input;
    // callers must use *_directed_weighted.
    let err = eigenvector_centrality_weighted(&g, &[1.0]);
    assert!(err.is_err());
}
