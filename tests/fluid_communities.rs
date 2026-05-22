//! Integration tests for `fluid_communities` /
//! `fluid_communities_with_options` (ALGO-CO-005).
//!
//! Exercises standard golden graphs (karate, ring-of-K5-cliques,
//! two-K4s + bridge), determinism under seed, and the documented error
//! paths.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]

use std::fs::File;
use std::path::PathBuf;

use rust_igraph::{
    FluidOptions, FluidResult, Graph, fluid_communities, fluid_communities_with_options,
    read_edgelist,
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

fn assert_partition_well_formed(r: &FluidResult, expected_n: usize) {
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
    assert_eq!(max_label + 1, r.nb_clusters, "nb_clusters not max(label)+1");
    let mut seen = vec![false; r.nb_clusters as usize];
    for &m in &r.membership {
        seen[m as usize] = true;
    }
    assert!(
        seen.into_iter().all(|b| b),
        "labels are not contiguous in [0, k)"
    );
}

#[test]
fn karate_k2_partition_well_formed() {
    let g = karate();
    let r = fluid_communities(&g, 2).unwrap();
    assert_partition_well_formed(&r, g.vcount() as usize);
    // Fluid with k=2 on karate routinely returns exactly 2 communities;
    // accept anything in [1, 2] in case a community vanishes.
    assert!((1..=2).contains(&r.nb_clusters));
}

#[test]
fn karate_k4_partition_well_formed() {
    let g = karate();
    let opts = FluidOptions {
        seed: 12345,
        ..FluidOptions::default()
    };
    let r = fluid_communities_with_options(&g, 4, &opts).unwrap();
    assert_partition_well_formed(&r, g.vcount() as usize);
    assert!((1..=4).contains(&r.nb_clusters));
}

#[test]
fn ring_of_4_cliques_finds_4_groups() {
    let g = ring_of_cliques(4, 5);
    let opts = FluidOptions {
        seed: 7,
        ..FluidOptions::default()
    };
    let r = fluid_communities_with_options(&g, 4, &opts).unwrap();
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
    let r = fluid_communities(&g, 2).unwrap();
    for u in 0..4 {
        assert_eq!(r.membership[u], r.membership[0]);
    }
    for u in 4..8 {
        assert_eq!(r.membership[u], r.membership[4]);
    }
    assert_ne!(r.membership[0], r.membership[4]);
}

#[test]
fn determinism_under_seed() {
    let g = karate();
    let opts = FluidOptions {
        seed: 999,
        ..FluidOptions::default()
    };
    let a = fluid_communities_with_options(&g, 3, &opts).unwrap();
    let b = fluid_communities_with_options(&g, 3, &opts).unwrap();
    assert_eq!(a.membership, b.membership);
    assert_eq!(a.nb_clusters, b.nb_clusters);
}

#[test]
fn k_equals_1_returns_single_community() {
    let g = karate();
    let r = fluid_communities(&g, 1).unwrap();
    assert_eq!(r.nb_clusters, 1);
    for &m in &r.membership {
        assert_eq!(m, 0);
    }
}

#[test]
fn k_equals_n_singletons_in_clique() {
    let mut g = Graph::with_vertices(5);
    for u in 0..5 {
        for v in (u + 1)..5 {
            g.add_edge(u, v).unwrap();
        }
    }
    let r = fluid_communities(&g, 5).unwrap();
    assert_partition_well_formed(&r, 5);
    assert_eq!(r.nb_clusters, 5);
}

#[test]
fn empty_graph_returns_empty() {
    let g = Graph::with_vertices(0);
    let r = fluid_communities(&g, 1).unwrap();
    assert_eq!(r.membership.len(), 0);
    assert_eq!(r.nb_clusters, 0);
}

#[test]
fn single_vertex_returns_single_community() {
    let g = Graph::with_vertices(1);
    let r = fluid_communities(&g, 1).unwrap();
    assert_eq!(r.membership.len(), 1);
    assert_eq!(r.nb_clusters, 1);
}

#[test]
fn rejects_k_zero() {
    let g = karate();
    assert!(fluid_communities(&g, 0).is_err());
}

#[test]
fn rejects_k_greater_than_vcount() {
    let mut g = Graph::with_vertices(4);
    for u in 0..4 {
        for v in (u + 1)..4 {
            g.add_edge(u, v).unwrap();
        }
    }
    assert!(fluid_communities(&g, 5).is_err());
}

#[test]
fn rejects_disconnected_graph() {
    let mut g = Graph::with_vertices(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(2, 3).unwrap();
    assert!(fluid_communities(&g, 2).is_err());
}

#[test]
fn rejects_non_simple_graph() {
    let mut g = Graph::with_vertices(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 1).unwrap(); // parallel
    assert!(fluid_communities(&g, 2).is_err());
}

#[test]
fn rejects_max_iterations_zero() {
    let g = karate();
    let opts = FluidOptions {
        seed: 0,
        max_iterations: 0,
    };
    assert!(fluid_communities_with_options(&g, 2, &opts).is_err());
}

#[test]
fn converges_in_iteration_cap() {
    let g = karate();
    let opts = FluidOptions {
        seed: 0,
        max_iterations: 1000,
    };
    let r = fluid_communities_with_options(&g, 3, &opts).unwrap();
    assert!(r.n_iterations_run >= 1);
    assert!(r.n_iterations_run <= 1000);
}
