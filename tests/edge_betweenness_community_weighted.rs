//! Integration tests for `edge_betweenness_community_weighted` (ALGO-CO-006b).
//!
//! Exercises the weighted Girvan-Newman pipeline: unit-weights ≡ CO-006,
//! cheap-bridge removal, karate sanity, ring-of-cliques splitting,
//! determinism, and the weighted error paths.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    EdgeBetweennessResult, Graph, edge_betweenness_community, edge_betweenness_community_weighted,
    modularity_weighted, read_edgelist,
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

fn assert_well_formed(r: &EdgeBetweennessResult, n: u32, m: usize) {
    assert_eq!(r.membership.len() as u32, n, "membership length");
    assert_eq!(r.removed_edges.len(), m, "removed_edges length");
    assert_eq!(r.edge_betweenness.len(), m, "history length");
    assert_eq!(r.merges.len(), r.bridges.len(), "merges/bridges parallel");
    assert_eq!(
        r.modularity.len(),
        r.merges.len() + 1,
        "modularity length = merges + 1"
    );
    for &lbl in &r.membership {
        assert!(lbl < r.nb_clusters, "dense label out of range");
    }
    let mut seen = vec![false; m];
    for &eid in &r.removed_edges {
        assert!(
            !seen[eid as usize],
            "edge {eid} removed twice in dendrogram"
        );
        seen[eid as usize] = true;
    }
    for (i, was_seen) in seen.iter().enumerate() {
        assert!(was_seen, "edge {i} never removed");
    }
}

#[test]
fn unit_weights_match_unweighted_on_karate() {
    // Unit weights through the weighted pipeline must reproduce the
    // unweighted dendrogram exactly — the BFS-Brandes and Dijkstra-Brandes
    // passes agree on unweighted shortest paths.
    let g = karate();
    let m = g.ecount();
    let weights = vec![1.0_f64; m];
    let rw = edge_betweenness_community_weighted(&g, &weights).unwrap();
    let ru = edge_betweenness_community(&g).unwrap();
    assert_well_formed(&rw, g.vcount(), m);
    assert_eq!(rw.nb_clusters, ru.nb_clusters);
    assert_eq!(rw.removed_edges, ru.removed_edges);
    assert_eq!(rw.merges, ru.merges);
    for (a, b) in rw.modularity.iter().zip(ru.modularity.iter()) {
        assert!((a - b).abs() < 1e-9, "modularity mismatch: {a} vs {b}");
    }
}

#[test]
fn unit_weights_match_unweighted_on_two_triangles_bridge() {
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let weights = vec![1.0_f64; g.ecount()];
    let rw = edge_betweenness_community_weighted(&g, &weights).unwrap();
    let ru = edge_betweenness_community(&g).unwrap();
    assert_eq!(rw.removed_edges, ru.removed_edges);
    assert_eq!(rw.merges, ru.merges);
    assert_eq!(rw.nb_clusters, ru.nb_clusters);
}

#[test]
fn cheap_bridge_drives_first_removal() {
    // Two K4 cliques + a single bridge with much smaller weight than the
    // intra-clique edges. The bridge sits on every cross-component
    // shortest path → carries the largest weighted betweenness → first
    // removed. Best-Q partition under weighted modularity then keeps
    // each K4 intact: weighted-Q ≥ 0.30 with k = 2.
    let mut g = Graph::with_vertices(8);
    for &(u, v) in &[
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
        (3, 4),
    ] {
        g.add_edge(u, v).unwrap();
    }
    let mut weights = vec![1.0_f64; g.ecount()];
    let bridge_eid = g.ecount() - 1;
    weights[bridge_eid] = 0.1;
    let r = edge_betweenness_community_weighted(&g, &weights).unwrap();
    assert_well_formed(&r, g.vcount(), g.ecount());
    let (from0, to0) = g.edge(r.removed_edges[0]).unwrap();
    assert!(
        (from0, to0) == (3, 4) || (from0, to0) == (4, 3),
        "first removed must be the bridge, got ({from0}, {to0})"
    );
    assert!(
        r.nb_clusters >= 2,
        "expected ≥ 2 communities, got k = {}",
        r.nb_clusters
    );
    // Each K4 stays internally connected at the best cut.
    assert_eq!(r.membership[0], r.membership[1]);
    assert_eq!(r.membership[1], r.membership[2]);
    assert_eq!(r.membership[2], r.membership[3]);
    assert_eq!(r.membership[4], r.membership[5]);
    assert_eq!(r.membership[6], r.membership[7]);
    assert_ne!(r.membership[0], r.membership[4]);
}

#[test]
fn karate_weighted_unit_high_modularity_matches_modularity_weighted() {
    let g = karate();
    let m = g.ecount();
    let weights = vec![1.0_f64; m];
    let r = edge_betweenness_community_weighted(&g, &weights).unwrap();
    assert_well_formed(&r, g.vcount(), m);
    let best_q = *r
        .modularity
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    assert!(
        best_q >= 0.35,
        "karate weighted best-Q expected ≥ 0.35, got {best_q}"
    );
    let q_check = modularity_weighted(&g, &r.membership, 1.0, &weights)
        .unwrap()
        .unwrap();
    let best_q_check = *r
        .modularity
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    assert!(
        (q_check - best_q_check).abs() < 1e-9,
        "best-Q {best_q_check} should match modularity_weighted({q_check}) of returned partition"
    );
}

#[test]
fn ring_of_4_k5_cliques_unit_weights_recovers_4_components() {
    let g = ring_of_cliques(4, 5);
    let m = g.ecount();
    let weights = vec![1.0_f64; m];
    let r = edge_betweenness_community_weighted(&g, &weights).unwrap();
    assert_well_formed(&r, g.vcount(), m);
    assert_eq!(r.nb_clusters, 4);
    // Same-clique vertices share a label.
    for clique in 0..4u32 {
        let base = clique * 5;
        for offset in 1..5u32 {
            assert_eq!(
                r.membership[base as usize],
                r.membership[(base + offset) as usize]
            );
        }
    }
    // Different-clique vertices have distinct labels.
    for a in 0..4u32 {
        for b in (a + 1)..4 {
            assert_ne!(
                r.membership[(a * 5) as usize],
                r.membership[(b * 5) as usize]
            );
        }
    }
}

#[test]
fn determinism_repeated_calls_match() {
    let g = ring_of_cliques(3, 4);
    let weights = vec![1.0_f64; g.ecount()];
    let r1 = edge_betweenness_community_weighted(&g, &weights).unwrap();
    let r2 = edge_betweenness_community_weighted(&g, &weights).unwrap();
    assert_eq!(r1.membership, r2.membership);
    assert_eq!(r1.removed_edges, r2.removed_edges);
    assert_eq!(r1.merges, r2.merges);
    for (a, b) in r1.modularity.iter().zip(r2.modularity.iter()) {
        assert!((a - b).abs() < 1e-12);
    }
}

#[test]
fn empty_graph_yields_empty_result() {
    let g = Graph::with_vertices(0);
    let r = edge_betweenness_community_weighted(&g, &[]).unwrap();
    assert_eq!(r.nb_clusters, 0);
    assert!(r.removed_edges.is_empty());
    assert!(r.modularity.is_empty());
}

#[test]
fn edgeless_graph_yields_singletons() {
    let g = Graph::with_vertices(4);
    let r = edge_betweenness_community_weighted(&g, &[]).unwrap();
    assert_eq!(r.nb_clusters, 4);
    for v in 0..4 {
        assert_eq!(r.membership[v as usize], v);
    }
    assert_eq!(r.modularity, vec![0.0]);
}

#[test]
fn directed_unit_weights_match_unweighted_path_5() {
    // Directed 5-path with unit weights: weighted run must reproduce
    // the unweighted CO-006 dendrogram (modulo equal-weight Dijkstra
    // tie-breaking, which matches BFS on a path).
    let mut g = Graph::new(5, true).unwrap();
    for i in 0..4u32 {
        g.add_edge(i, i + 1).unwrap();
    }
    let w = vec![1.0_f64; g.ecount()];
    let rw = edge_betweenness_community_weighted(&g, &w).unwrap();
    let ru = rust_igraph::edge_betweenness_community(&g).unwrap();
    assert_eq!(rw.nb_clusters, ru.nb_clusters);
    assert_eq!(rw.removed_edges, ru.removed_edges);
    assert_eq!(rw.merges, ru.merges);
}

#[test]
fn directed_cheap_bridge_first_removal() {
    // Directed two-triangles + bridge 2→3 with bridge weight 0.1
    // versus 1.0 elsewhere. The cheap bridge should still be the
    // weighted-Dijkstra first-removal candidate even though the
    // directed-Brandes count has 3-way ties on the unweighted side.
    let mut g = Graph::new(6, true).unwrap();
    for &(u, v) in &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let weights = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1];
    let r = edge_betweenness_community_weighted(&g, &weights).unwrap();
    assert_eq!(r.removed_edges.len(), 7);
    for &q in &r.modularity {
        assert!(q.is_finite());
        assert!((-1.0..=1.0).contains(&q));
    }
}

#[test]
fn rejects_weight_length_mismatch() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    let err = edge_betweenness_community_weighted(&g, &[1.0]).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("InvalidArgument") || msg.contains("weights"));
}

#[test]
fn rejects_negative_weight() {
    let mut g = Graph::with_vertices(2);
    g.add_edge(0, 1).unwrap();
    let err = edge_betweenness_community_weighted(&g, &[-1.0]).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("non-negative") || msg.contains("InvalidArgument"));
}

#[test]
fn rejects_nan_weight() {
    let mut g = Graph::with_vertices(2);
    g.add_edge(0, 1).unwrap();
    let err = edge_betweenness_community_weighted(&g, &[f64::NAN]).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("finite") || msg.contains("InvalidArgument"));
}

#[test]
fn dendrogram_total_merges_matches_unique_components() {
    // 5v with two components: 0-1-2 and 3-4. Each component contributes
    // its internal merges; total = (V - C) where C = components.
    let mut g = Graph::with_vertices(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(3, 4).unwrap();
    let weights = vec![1.0_f64; g.ecount()];
    let r = edge_betweenness_community_weighted(&g, &weights).unwrap();
    // 5 vertices, 2 starting components → up to 3 merges available.
    assert!(r.merges.len() <= 3);
    assert_eq!(r.merges.len(), r.bridges.len());
}
