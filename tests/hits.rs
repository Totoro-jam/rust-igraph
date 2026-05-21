//! Integration tests for `hub_and_authority_scores` (ALGO-PR-017).
//!
//! Cross the public API surface and exercise scenarios that the
//! in-module unit tests can't: larger graphs, the karate-club fixture
//! we ship for cross-algorithm parity, and the pull-back invariants
//! between hub, authority, and the eigenvalue.

use rust_igraph::{Graph, hub_and_authority_scores};

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
fn directed_path_chain_all_intermediate_zero_loop() {
    // 0 → 1 → 2 → 3: pure feedforward path.
    // hub_3 = 0 (sink); authority_0 = 0 (source).
    let mut g = Graph::new(4, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (1, 2), (2, 3)]).unwrap();
    let s = hub_and_authority_scores(&g).unwrap();
    assert!(s.hub[3].abs() < 1e-9);
    assert!(s.authority[0].abs() < 1e-9);
    // 0, 1, 2 each have exactly one out-edge to a unique authority,
    // so all three are equally good hubs after normalisation.
    close(&s.hub[..3], &[1.0, 1.0, 1.0], 1e-9);
    close(&s.authority[1..], &[1.0, 1.0, 1.0], 1e-9);
}

#[test]
#[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
fn cross_relation_h_eq_a_authority_normalised() {
    // After convergence, h ∝ A·authority (max-normalised both sides).
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
    let s = hub_and_authority_scores(&g).unwrap();

    let n = g.vcount();
    let mut a_auth = vec![0.0_f64; n as usize];
    // Scan the edge list once: a_auth[u] = Σ_{v ∈ out(u)} authority[v].
    for e in 0..g.ecount() {
        let (u, v) = g.edge(e as u32).unwrap();
        a_auth[u as usize] += s.authority[v as usize];
    }
    let max = a_auth.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
    if max > 0.0 {
        for slot in &mut a_auth {
            *slot /= max;
        }
    }
    for (u, &val) in a_auth.iter().enumerate() {
        assert!(
            (val - s.hub[u]).abs() < 1e-6,
            "vertex {u}: A·a={val} hub={}",
            s.hub[u]
        );
    }
}

#[test]
#[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
fn cross_relation_a_eq_at_hub_normalised() {
    // After convergence, authority ∝ Aᵀ·hub.
    let mut g = Graph::new(5, true).unwrap();
    g.add_edges(vec![(0u32, 1u32), (0, 2), (1, 3), (2, 3), (3, 4), (4, 0)])
        .unwrap();
    let s = hub_and_authority_scores(&g).unwrap();

    let n = g.vcount();
    let mut at_hub = vec![0.0_f64; n as usize];
    for e in 0..g.ecount() {
        let (u, v) = g.edge(e as u32).unwrap();
        at_hub[v as usize] += s.hub[u as usize];
    }
    let max = at_hub.iter().fold(0.0_f64, |acc, &x| acc.max(x.abs()));
    if max > 0.0 {
        for slot in &mut at_hub {
            *slot /= max;
        }
    }
    close(&at_hub, &s.authority, 1e-6);
}

#[test]
fn karate_undirected_matches_eigenvector() {
    // On the undirected Zachary karate club, hub == auth == eigenvector
    // centrality (delegation path).
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
    let s = hub_and_authority_scores(&g).unwrap();
    let ec = rust_igraph::eigenvector_centrality(&g).unwrap();
    close(&s.hub, &ec, 1e-9);
    close(&s.authority, &ec, 1e-9);
    // Eigenvalue should be the squared dominant eigenvalue of A;
    // simply assert positive and finite.
    assert!(s.eigenvalue > 0.0 && s.eigenvalue.is_finite());
}

#[test]
fn no_negative_components() {
    // For a non-negative weight regime (here unweighted), no entry
    // should be negative even after normalisation drift.
    let mut g = Graph::new(8, true).unwrap();
    for u in 0..7u32 {
        g.add_edge(u, u + 1).unwrap();
    }
    g.add_edge(7, 0).unwrap();
    let s = hub_and_authority_scores(&g).unwrap();
    for &x in &s.hub {
        assert!(x >= 0.0, "hub had negative {x}");
    }
    for &x in &s.authority {
        assert!(x >= 0.0, "authority had negative {x}");
    }
}

#[test]
fn empty_directed_no_edges_returns_ones_zero_eigenvalue() {
    let g = Graph::new(5, true).unwrap();
    let s = hub_and_authority_scores(&g).unwrap();
    close(&s.hub, &[1.0; 5], 1e-15);
    close(&s.authority, &[1.0; 5], 1e-15);
    assert!(s.eigenvalue.abs() < f64::EPSILON);
}
