//! Integration tests for `eigenvector_centrality_directed` (ALGO-PR-012b).
//!
//! Covers mode dispatch (Out / In / All), DAG sentinels, the upstream
//! cycle+chord golden, and the master `_full` entry point.

use rust_igraph::{
    EigenvectorMode, Graph, eigenvector_centrality_directed,
    eigenvector_centrality_directed_weighted, eigenvector_centrality_full,
};

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
fn directed_k5_complete_eigenvalue_four() {
    let mut g = Graph::new(5, true).unwrap();
    for u in 0..5u32 {
        for v in 0..5u32 {
            if u != v {
                g.add_edge(u, v).unwrap();
            }
        }
    }
    let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
    close(&s.vector, &[1.0; 5], 1e-9);
    assert!((s.eigenvalue - 4.0).abs() < 1e-9);
}

#[test]
fn directed_in_mode_matches_out_on_symmetric_topology() {
    // Bidirectional digraph (every undirected edge replaced by two
    // arcs) — In and Out modes must agree.
    let mut g = Graph::new(4, true).unwrap();
    for u in 0..4u32 {
        for v in 0..4u32 {
            if u != v {
                g.add_edge(u, v).unwrap();
            }
        }
    }
    let out = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
    let in_ = eigenvector_centrality_directed(&g, EigenvectorMode::In).unwrap();
    close(&out.vector, &in_.vector, 1e-9);
    assert!((out.eigenvalue - in_.eigenvalue).abs() < 1e-9);
}

#[test]
fn directed_cycle_chord_eigenvalue_upstream_golden() {
    // 0→1→2→3→0 plus 1→3. λ ≈ 1.22074, vec[3]=1 max.
    let mut g = Graph::new(4, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 0), (1, 3)])
        .unwrap();
    let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
    close(
        &s.vector,
        &[0.819_172_5, 0.671_043_6, 0.549_700_5, 1.0],
        1e-4,
    );
    assert!((s.eigenvalue - 1.220_744_084_6).abs() < 1e-9);
}

#[test]
fn directed_dag_out_star_marks_leaves() {
    let mut g = Graph::new(5, true).unwrap();
    for v in 1..5u32 {
        g.add_edge(0, v).unwrap();
    }
    let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
    close(&s.vector, &[0.0, 1.0, 1.0, 1.0, 1.0], 1e-12);
    assert!(s.eigenvalue.abs() < f64::EPSILON);
}

#[test]
fn directed_dag_in_star_out_mode_marks_root_only() {
    // 1,2,3,4 → 0. Sinks (no out edges) = {0}. Out mode returns
    // [1,0,0,0,0].
    let mut g = Graph::new(5, true).unwrap();
    for v in 1..5u32 {
        g.add_edge(v, 0).unwrap();
    }
    let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
    close(&s.vector, &[1.0, 0.0, 0.0, 0.0, 0.0], 1e-12);
    assert!(s.eigenvalue.abs() < f64::EPSILON);
}

#[test]
fn directed_empty_edges_returns_uniform_one() {
    let g = Graph::new(5, true).unwrap();
    let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
    close(&s.vector, &[1.0; 5], 1e-12);
}

#[test]
fn directed_weighted_unit_matches_unweighted() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3), (3, 0), (1, 3)])
        .unwrap();
    let unw = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
    let w =
        eigenvector_centrality_directed_weighted(&g, EigenvectorMode::Out, &vec![1.0; g.ecount()])
            .unwrap();
    close(&w.vector, &unw.vector, 1e-9);
    assert!((w.eigenvalue - unw.eigenvalue).abs() < 1e-9);
}

#[test]
fn directed_full_dispatches_to_undirected_path_on_undirected_input() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();
    let a = eigenvector_centrality_full(&g, EigenvectorMode::Out, None).unwrap();
    let b = eigenvector_centrality_full(&g, EigenvectorMode::All, None).unwrap();
    close(&a.vector, &b.vector, 1e-12);
    assert!((a.eigenvalue - b.eigenvalue).abs() < 1e-12);
}

#[test]
fn directed_full_weights_length_mismatch_errors() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let err = eigenvector_centrality_full(&g, EigenvectorMode::Out, Some(&[1.0]));
    assert!(err.is_err());
}

#[test]
fn directed_cycle_100_circulant_uniform() {
    // Directed cycle of 100 — adjacency eigenvalues are 100th roots of
    // unity, dominant λ=1, uniform eigenvector.
    let n: u32 = 100;
    let mut g = Graph::new(n, true).unwrap();
    for i in 0..n {
        g.add_edge(i, (i + 1) % n).unwrap();
    }
    let s = eigenvector_centrality_directed(&g, EigenvectorMode::Out).unwrap();
    assert!((s.eigenvalue - 1.0).abs() < 1e-9);
    let mx = s.vector.iter().copied().fold(0.0_f64, f64::max);
    let mn = s.vector.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(mx - mn < 1e-9, "vec spread {} too large", mx - mn);
}
