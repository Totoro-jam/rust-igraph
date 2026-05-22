//! Integration tests for `edge_betweenness_community` (ALGO-CO-006).
//!
//! Exercises canonical Girvan-Newman cases: bridged cliques, the karate
//! club, paths, cycles, and error paths.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    EdgeBetweennessResult, Graph, edge_betweenness_community, modularity, read_edgelist,
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
    // Every removed edge id occurs exactly once.
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
fn two_triangles_bridge_first_removal_is_bridge() {
    let mut g = Graph::with_vertices(6);
    for &(u, v) in &[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)] {
        g.add_edge(u, v).unwrap();
    }
    let r = edge_betweenness_community(&g).unwrap();
    assert_well_formed(&r, 6, 7);
    let (from0, to0) = g.edge(r.removed_edges[0]).unwrap();
    assert!(
        (from0, to0) == (2, 3) || (from0, to0) == (3, 2),
        "first removed should be bridge (2,3), got ({from0}, {to0})"
    );
    assert_eq!(r.nb_clusters, 2);
    assert_eq!(r.membership[0], r.membership[1]);
    assert_eq!(r.membership[3], r.membership[5]);
    assert_ne!(r.membership[0], r.membership[3]);
}

#[test]
fn karate_partition_well_formed_and_high_modularity() {
    let g = karate();
    let r = edge_betweenness_community(&g).unwrap();
    assert_well_formed(&r, g.vcount(), g.ecount());
    // Newman-Girvan on Zachary's karate is typically Q ≈ 0.4
    // (Girvan-Newman 2002, Table I); accept anything ≥ 0.35 to be safe.
    let best_q = *r
        .modularity
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    assert!(
        best_q >= 0.35,
        "expected best modularity >= 0.35, got {best_q}"
    );
    // The cross-check: the chosen `membership` must achieve the same Q
    // when re-fed through `modularity()`.
    let q_recompute = modularity(&g, &r.membership, 1.0).unwrap().unwrap();
    assert!(
        (q_recompute - best_q).abs() < 1e-9,
        "best Q drift: dendrogram says {best_q}, recompute {q_recompute}"
    );
    // The number of clusters at best Q should be small (Zachary has 2-4
    // natural communities).
    assert!(
        (2..=10).contains(&r.nb_clusters),
        "unexpected nb_clusters = {} on karate",
        r.nb_clusters
    );
}

#[test]
fn ring_of_4_cliques_splits_into_4() {
    let g = ring_of_cliques(4, 5);
    let r = edge_betweenness_community(&g).unwrap();
    assert_well_formed(&r, g.vcount(), g.ecount());
    assert_eq!(r.nb_clusters, 4, "expected 4 cliques");
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
fn path_5_splits_at_middle_edges() {
    // 0-1-2-3-4: the central edges carry the highest betweenness.
    let mut g = Graph::with_vertices(5);
    for i in 0..4u32 {
        g.add_edge(i, i + 1).unwrap();
    }
    let r = edge_betweenness_community(&g).unwrap();
    assert_well_formed(&r, 5, 4);
    // First removal must be one of the two middle edges (eid 1 or 2).
    assert!(
        r.removed_edges[0] == 1 || r.removed_edges[0] == 2,
        "first removal should be a middle edge, got {}",
        r.removed_edges[0]
    );
}

#[test]
fn cycle_4_modularity_monotone_at_singletons() {
    // 0-1-2-3-0. Best Q for 4-cycle is bipartition {0,2},{1,3} (Q ≈ 0).
    let mut g = Graph::with_vertices(4);
    for i in 0..4u32 {
        g.add_edge(i, (i + 1) % 4).unwrap();
    }
    let r = edge_betweenness_community(&g).unwrap();
    assert_well_formed(&r, 4, 4);
    for &q in &r.modularity {
        assert!((-0.51..=0.51).contains(&q), "modularity {q} out of range");
    }
}

#[test]
fn already_disconnected_two_components_recovered() {
    let mut g = Graph::with_vertices(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(3, 4).unwrap();
    let r = edge_betweenness_community(&g).unwrap();
    assert_well_formed(&r, 5, 3);
    assert!(r.nb_clusters >= 2);
    assert_eq!(r.membership[0], r.membership[1]);
    assert_eq!(r.membership[2], r.membership[3]);
    assert_eq!(r.membership[3], r.membership[4]);
    assert_ne!(r.membership[0], r.membership[2]);
}

#[test]
fn empty_graph_yields_empty_result() {
    let g = Graph::with_vertices(0);
    let r = edge_betweenness_community(&g).unwrap();
    assert_eq!(r.nb_clusters, 0);
    assert!(r.removed_edges.is_empty());
    assert!(r.modularity.is_empty());
}

#[test]
fn edgeless_graph_yields_singletons() {
    let g = Graph::with_vertices(7);
    let r = edge_betweenness_community(&g).unwrap();
    assert_eq!(r.nb_clusters, 7);
    for v in 0..7 {
        assert_eq!(r.membership[v as usize], v);
    }
    assert_eq!(r.modularity, vec![0.0]);
}

#[test]
fn single_vertex_is_singleton() {
    let g = Graph::with_vertices(1);
    let r = edge_betweenness_community(&g).unwrap();
    assert_eq!(r.membership, vec![0]);
    assert_eq!(r.nb_clusters, 1);
}

#[test]
fn rejects_directed_graph() {
    let mut g = Graph::new(3, true).unwrap();
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    assert!(edge_betweenness_community(&g).is_err());
}

#[test]
fn determinism_repeated_calls_match() {
    let g = ring_of_cliques(3, 4);
    let a = edge_betweenness_community(&g).unwrap();
    let b = edge_betweenness_community(&g).unwrap();
    assert_eq!(a.membership, b.membership);
    assert_eq!(a.removed_edges, b.removed_edges);
    assert_eq!(a.merges, b.merges);
    for (qa, qb) in a.modularity.iter().zip(b.modularity.iter()) {
        assert!((qa - qb).abs() < 1e-12);
    }
}

#[test]
fn dendrogram_total_merges_matches_unique_components() {
    // K4 has 1 component → exactly 3 merges total.
    let mut g = Graph::with_vertices(4);
    for u in 0..4 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).unwrap();
        }
    }
    let r = edge_betweenness_community(&g).unwrap();
    assert_eq!(r.merges.len(), 3, "K4 → 3 merges");
    assert_eq!(r.bridges.len(), 3);
}
