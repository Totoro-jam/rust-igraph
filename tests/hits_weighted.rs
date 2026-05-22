//! Integration tests for `hub_and_authority_scores_weighted` (ALGO-PR-017b).
//!
//! Mirrors the unweighted integration suite (`tests/hits.rs`) but
//! exercises the weighted matrix `W[i,j] = Σ_{e: i→j} w_e`. Covers
//! length validation, the cross-relation `hub ∝ W·authority`, parity
//! with the unweighted path under unit weights, and the empty/all-zero
//! sentinel cases.

use rust_igraph::{Graph, hub_and_authority_scores, hub_and_authority_scores_weighted};

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
fn weighted_unit_matches_unweighted_directed_chain() {
    let mut g = Graph::new(6, true).unwrap();
    g.add_edges(vec![
        (0u32, 1u32),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 5),
        (0, 5),
        (2, 5),
    ])
    .unwrap();
    let unweighted = hub_and_authority_scores(&g).unwrap();
    let weighted = hub_and_authority_scores_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
    close(&weighted.hub, &unweighted.hub, 1e-6);
    close(&weighted.authority, &unweighted.authority, 1e-6);
    assert!((weighted.eigenvalue - unweighted.eigenvalue).abs() < 1e-6);
}

#[test]
#[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
fn weighted_cross_relation_h_eq_w_authority() {
    // After convergence, h ∝ W·authority (max-normalised both sides).
    let mut g = Graph::new(6, true).unwrap();
    g.add_edges(vec![
        (0u32, 1u32),
        (0, 2),
        (1, 3),
        (2, 4),
        (3, 5),
        (4, 5),
        (1, 5),
    ])
    .unwrap();
    let weights = vec![1.5, 2.5, 0.75, 1.0, 0.25, 2.0, 1.25];
    let s = hub_and_authority_scores_weighted(&g, &weights).unwrap();

    let n = g.vcount();
    let mut w_auth = vec![0.0_f64; n as usize];
    for (e, &w) in weights.iter().enumerate() {
        let (u, v) = g.edge(e as u32).unwrap();
        w_auth[u as usize] += w * s.authority[v as usize];
    }
    let max = w_auth.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
    if max > 0.0 {
        for slot in &mut w_auth {
            *slot /= max;
        }
    }
    for (u, &val) in w_auth.iter().enumerate() {
        assert!(
            (val - s.hub[u]).abs() < 1e-6,
            "vertex {u}: W·a={val} hub={}",
            s.hub[u]
        );
    }
}

#[test]
#[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
fn weighted_cross_relation_a_eq_wt_hub() {
    // After convergence, authority ∝ Wᵀ·hub.
    let mut g = Graph::new(5, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (2, 3), (3, 4), (4, 0)])
        .unwrap();
    let weights = vec![1.5, 0.5, 2.0, 1.0, 0.75, 1.25];
    let s = hub_and_authority_scores_weighted(&g, &weights).unwrap();

    let n = g.vcount();
    let mut wt_hub = vec![0.0_f64; n as usize];
    for (e, &w) in weights.iter().enumerate() {
        let (u, v) = g.edge(e as u32).unwrap();
        wt_hub[v as usize] += w * s.hub[u as usize];
    }
    let max = wt_hub.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
    if max > 0.0 {
        for slot in &mut wt_hub {
            *slot /= max;
        }
    }
    close(&wt_hub, &s.authority, 1e-6);
}

#[test]
fn weighted_karate_undirected_matches_unweighted_under_unit_weights() {
    // Undirected Zachary karate club under unit weights ⇒ same as
    // unweighted delegation path.
    let edges_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/karate.edges");
    let raw = std::fs::read_to_string(&edges_path).expect("read karate.edges");
    let mut edges = Vec::new();
    let mut max_v = 0u32;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.split_whitespace();
        let u: u32 = it.next().unwrap().parse().unwrap();
        let v: u32 = it.next().unwrap().parse().unwrap();
        max_v = max_v.max(u).max(v);
        edges.push((u, v));
    }
    let mut g = Graph::with_vertices(max_v + 1);
    for (u, v) in edges {
        g.add_edge(u, v).unwrap();
    }
    let unweighted = hub_and_authority_scores(&g).unwrap();
    let weighted = hub_and_authority_scores_weighted(&g, &vec![1.0; g.ecount()]).unwrap();
    close(&weighted.hub, &unweighted.hub, 1e-6);
    close(&weighted.authority, &unweighted.authority, 1e-6);
    assert!(weighted.eigenvalue > 0.0 && weighted.eigenvalue.is_finite());
}

#[test]
fn weighted_length_mismatch_invalid_argument() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2)]).unwrap();
    let s = hub_and_authority_scores_weighted(&g, &[1.0]);
    assert!(s.is_err(), "expected error for wrong-length weights");
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn weighted_no_negative_components_for_positive_weights() {
    // For positive-only weights, sign-cleanup should keep all entries
    // non-negative even after normalisation drift.
    let mut g = Graph::new(8, true).unwrap();
    for u in 0..7u32 {
        g.add_edge(u, u + 1).unwrap();
    }
    g.add_edge(7, 0).unwrap();
    let weights = (1..=g.ecount()).map(|i| i as f64).collect::<Vec<_>>();
    let s = hub_and_authority_scores_weighted(&g, &weights).unwrap();
    for &x in &s.hub {
        assert!(x >= 0.0, "hub had negative {x}");
    }
    for &x in &s.authority {
        assert!(x >= 0.0, "authority had negative {x}");
    }
}

#[test]
fn weighted_empty_directed_no_edges_returns_ones_zero_eigenvalue() {
    let g = Graph::new(5, true).unwrap();
    let s = hub_and_authority_scores_weighted(&g, &[]).unwrap();
    close(&s.hub, &[1.0; 5], 1e-15);
    close(&s.authority, &[1.0; 5], 1e-15);
    assert!(s.eigenvalue.abs() < f64::EPSILON);
}

#[test]
fn weighted_all_zero_returns_ones_zero_eigenvalue() {
    let mut g = Graph::new(4, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
    let s = hub_and_authority_scores_weighted(&g, &[0.0, 0.0, 0.0]).unwrap();
    close(&s.hub, &[1.0; 4], 1e-15);
    close(&s.authority, &[1.0; 4], 1e-15);
    assert!(s.eigenvalue.abs() < f64::EPSILON);
}
