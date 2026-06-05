//! HRG cross-validation tests against igraph C deterministic test cases.
//!
//! Based on `references/igraph/tests/unit/igraph_hrg_create.c` and
//! `references/igraph/tests/unit/igraph_hrg3.c`.
//!
//! Stochastic HRG functions (`hrg_fit`, `hrg_predict`, `hrg_consensus`) cannot be
//! exactly matched across implementations due to RNG differences. Instead we
//! test deterministic scenarios (prob=0.0 or prob=1.0) and statistical
//! properties.

#![allow(clippy::float_cmp)]

use rust_igraph::{
    Graph, HrgTree, from_hrg_dendrogram, hrg_create, hrg_fit, hrg_game, hrg_predict, hrg_sample,
};

/// igraph C test: Two leaf nodes with prob=1.0 always produces edge 0-1.
/// Source: `igraph_hrg_create.c` line 30-39
#[test]
fn hrg_create_two_leaves_prob_one_deterministic() {
    // Dendrogram: vertex 0 (internal) -> vertices 1, 2 (leaves)
    let g = Graph::from_edges(&[(0, 1), (0, 2)], true, None).expect("create dendrogram");
    let prob = vec![1.0];
    let hrg = hrg_create(&g, &prob).expect("hrg_create");

    assert_eq!(hrg.size(), 2);
    assert_eq!(hrg.num_internal(), 1);
    assert!((hrg.prob[0] - 1.0).abs() < 1e-10);

    // With prob=1.0, generated graph always has edge between the two leaves
    let sampled = hrg_game(&hrg, 42).expect("hrg_game");
    assert_eq!(sampled.vcount(), 2);
    assert_eq!(sampled.ecount(), 1);

    // Verify determinism: any seed produces the same result
    for seed in [0, 1, 999, u64::MAX] {
        let g2 = hrg_game(&hrg, seed).expect("hrg_game");
        assert_eq!(g2.ecount(), 1);
    }
}

/// igraph C test: Four leaf nodes with probs [1.0, 0.0, 0.0].
/// One vertex connected to all others (root prob=1.0), no intra-subtree
/// connections (sub-tree probs=0.0).
/// Source: `igraph_hrg_create.c` line 43-47
///
/// Expected: 4 vertices, 3 edges — one leaf connects to all 3 others.
#[test]
fn hrg_create_four_leaves_mixed_probs_deterministic() {
    // Dendrogram tree: 7 vertices
    // Vertex 0 (internal root) -> 3 (leaf), 1 (internal)
    // Vertex 1 (internal) -> 4 (leaf), 2 (internal)
    // Vertex 2 (internal) -> 5 (leaf), 6 (leaf)
    let g = Graph::from_edges(
        &[(0, 3), (0, 1), (1, 4), (1, 2), (2, 5), (2, 6)],
        true,
        None,
    )
    .expect("create dendrogram");

    let prob = vec![1.0, 0.0, 0.0];
    let hrg = hrg_create(&g, &prob).expect("hrg_create");

    assert_eq!(hrg.size(), 4);
    assert_eq!(hrg.num_internal(), 3);

    // Generate graph — deterministic since probs are 0 or 1
    let sampled = hrg_game(&hrg, 42).expect("hrg_game");
    assert_eq!(sampled.vcount(), 4);
    assert_eq!(sampled.ecount(), 3);

    // One vertex should have degree 3 (connected to all others)
    let degrees: Vec<usize> = (0..4).map(|v| sampled.degree(v).expect("degree")).collect();
    assert!(
        degrees.contains(&3),
        "Expected one vertex with degree 3, got {degrees:?}"
    );

    // The other three vertices should each have degree 1
    let deg1_count = degrees.iter().filter(|&&d| d == 1).count();
    assert_eq!(deg1_count, 3);

    // Verify determinism across seeds
    for seed in [0, 123, 9999] {
        let g2 = hrg_game(&hrg, seed).expect("hrg_game");
        assert_eq!(g2.ecount(), 3);
    }
}

/// igraph C test: prob=0.0 produces empty graph (no edges).
#[test]
fn hrg_create_prob_zero_no_edges() {
    // Simple 2-leaf tree with prob=0.0
    let g = Graph::from_edges(&[(0, 1), (0, 2)], true, None).expect("create dendrogram");
    let prob = vec![0.0];
    let hrg = hrg_create(&g, &prob).expect("hrg_create");

    // No edges should ever appear
    for seed in [0, 42, 12345, u64::MAX] {
        let sampled = hrg_game(&hrg, seed).expect("hrg_game");
        assert_eq!(sampled.vcount(), 2);
        assert_eq!(sampled.ecount(), 0);
    }
}

/// igraph C test: `from_hrg_dendrogram` round-trip.
/// Build an HRG, convert to dendrogram, verify structure.
#[test]
fn from_hrg_dendrogram_structure() {
    // Build a 3-leaf HRG manually
    let mut hrg = HrgTree::new(3);
    hrg.left[0] = 0; // root's left child = leaf 0
    hrg.right[0] = -2; // root's right child = internal node 1
    hrg.prob[0] = 0.5;
    hrg.left[1] = 1; // internal 1's left child = leaf 1
    hrg.right[1] = 2; // internal 1's right child = leaf 2
    hrg.prob[1] = 0.8;
    hrg.vertices = vec![3, 2];
    hrg.edges = vec![4, 2];

    let d = from_hrg_dendrogram(&hrg).expect("from_hrg_dendrogram");

    // 5 vertices total: 3 leaves + 2 internal
    assert_eq!(d.graph.vcount(), 5);
    // 4 directed edges (each internal has 2 children)
    assert_eq!(d.graph.ecount(), 4);
    // Leaf probabilities are NAN, internal are set
    assert!(d.prob[0].is_nan());
    assert!(d.prob[1].is_nan());
    assert!(d.prob[2].is_nan());
    assert!((d.prob[3] - 0.5).abs() < 1e-10);
    assert!((d.prob[4] - 0.8).abs() < 1e-10);
}

/// igraph C test: `hrg_create` error — wrong number of probabilities.
/// Source: `igraph_hrg_create.c` line 56-58
#[test]
fn hrg_create_error_wrong_prob_count() {
    // 3-vertex tree has 1 internal node, but we pass 3 probs
    let g = Graph::from_edges(&[(0, 1), (0, 2)], true, None).expect("create dendrogram");
    let prob = vec![1.0, 0.0, 0.0]; // wrong: should be length 1
    let result = hrg_create(&g, &prob);
    assert!(result.is_err(), "Should error on wrong prob count");
}

/// igraph C test: `hrg_fit` requires at least 3 vertices.
/// Source: `igraph_hrg.c` line 94-98
#[test]
fn hrg_fit_error_too_few_vertices() {
    let g = Graph::from_edges(&[(0, 1)], false, None).expect("create K2");
    assert_eq!(g.vcount(), 2);
    let result = hrg_fit(&g, None, 100, 42);
    assert!(result.is_err(), "hrg_fit should require >= 3 vertices");
}

/// igraph C test: `hrg_predict` probabilities are all in [0, 1].
/// Source: `igraph_hrg3.c` line 72-89
#[test]
fn hrg_predict_probabilities_valid_range() {
    // Small graph with clear community structure
    let g = Graph::from_edges(
        &[(0, 1), (0, 2), (1, 2), (2, 3), (3, 4), (3, 5), (4, 5)],
        false,
        None,
    )
    .expect("create graph");

    // hrg_predict returns Vec<(u32, u32, f64)> — sparse edge predictions
    let predictions = hrg_predict(&g, None, 50, 42).expect("hrg_predict");

    let n = g.vcount();

    // All probabilities should be in [0, 1]
    for &(u, v, p) in &predictions {
        assert!(
            (0.0..=1.0).contains(&p),
            "Probability {p} out of range [0,1] for edge ({u}, {v})"
        );
        // No self-loops
        assert_ne!(u, v, "Self-loop prediction should not appear");
        // Valid vertex ids
        assert!(u < n && v < n, "Vertex id out of range: ({u}, {v}), n={n}");
    }

    // Check symmetry: for every (u, v, p), there should be (v, u, p)
    for &(u, v, p) in &predictions {
        let mirror = predictions.iter().find(|&&(a, b, _)| a == v && b == u);
        if let Some(&(_, _, p2)) = mirror {
            assert!(
                (p - p2).abs() < 1e-10,
                "Prediction not symmetric: ({u},{v})={p} vs ({v},{u})={p2}"
            );
        }
    }
}

/// Statistical property: `hrg_sample` with prob=1.0 everywhere produces
/// complete graph.
#[test]
fn hrg_sample_all_prob_one_produces_complete_graph() {
    let mut hrg = HrgTree::new(4);
    // Root: left=0, right=internal1
    hrg.left[0] = 0;
    hrg.right[0] = -2;
    hrg.prob[0] = 1.0;
    // Internal1: left=1, right=internal2
    hrg.left[1] = 1;
    hrg.right[1] = -3;
    hrg.prob[1] = 1.0;
    // Internal2: left=2, right=3
    hrg.left[2] = 2;
    hrg.right[2] = 3;
    hrg.prob[2] = 1.0;
    hrg.vertices = vec![4, 3, 2];
    hrg.edges = vec![6, 4, 2];

    // With all probs=1.0, every pair of leaves connects → K4 has 6 edges
    let g = hrg_sample(&hrg, 42).expect("hrg_sample");
    assert_eq!(g.vcount(), 4);
    assert_eq!(g.ecount(), 6); // C(4,2) = 6

    // Any seed gives the same result
    let g2 = hrg_sample(&hrg, 0).expect("hrg_sample");
    assert_eq!(g2.ecount(), 6);
}

/// Statistical property: `hrg_sample` with prob=0.0 everywhere produces
/// empty graph (no edges).
#[test]
fn hrg_sample_all_prob_zero_produces_empty_graph() {
    let mut hrg = HrgTree::new(4);
    hrg.left[0] = 0;
    hrg.right[0] = -2;
    hrg.prob[0] = 0.0;
    hrg.left[1] = 1;
    hrg.right[1] = -3;
    hrg.prob[1] = 0.0;
    hrg.left[2] = 2;
    hrg.right[2] = 3;
    hrg.prob[2] = 0.0;
    hrg.vertices = vec![4, 3, 2];
    hrg.edges = vec![6, 4, 2];

    let g = hrg_sample(&hrg, 42).expect("hrg_sample");
    assert_eq!(g.vcount(), 4);
    assert_eq!(g.ecount(), 0);
}
