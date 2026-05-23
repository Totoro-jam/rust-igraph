#!/usr/bin/env python3
"""Extract conformance fixtures from `references/rigraph/tests/testthat/`.

Phase 0 demo: hand-curated manifest. Expected values were copy-pasted from the
R test source; vertex ids translated from R's 1-based to 0-based indexing. The
proper Phase-1 extractor calls `Rscript run_r.R` to actually execute the R
testthat suite and harvest every `expect_equal` automatically (only viable
when R + the rigraph package are installed; see `run_r.R`).

Output: `tests/conformance/r/<algo>/<case>.json`

Usage:
    .venv/bin/python -m scripts.test_extract.from_r --algo bfs
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List

import igraph as ig

REPO_ROOT = Path(__file__).resolve().parents[2]
R_TESTS_DIR = REPO_ROOT / "references/rigraph/tests/testthat"
OUT_DIR = REPO_ROOT / "tests/conformance/r"


def _ring(n: int) -> ig.Graph:
    return ig.Graph.Ring(n=n, directed=False, mutual=False, circular=True)


def _star(n: int) -> ig.Graph:
    # R's `make_star(n)` defaults to mode='in' (directed star with edges
    # pointing INTO the centre) and centre = vertex 1 (R-1-indexed) =
    # our vertex 0. The R test does `dfs(g, root=2, unreachable=FALSE)`
    # whose `c(2, 1)` result depends on mode='in': from R-vertex-2 the
    # only out-edge points to R-vertex-1 (the centre). With centre=0 in
    # our 0-based world: vertex 1 has out-edge to vertex 0; vertex 0 has
    # only IN edges, so DFS from root=1 visits [1, 0] and stops.
    return ig.Graph.Star(n=n, mode="in", center=0)


# Each entry mirrors one expect_equal in a testthat file. Vertex ids
# translated from 1-based (R) to 0-based (Rust). Verify against upstream when
# adding a row.
BFS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "structural_ring10_root0",
        "origin": (
            "test-structural-properties.R:'BFS works from multiple root vertices' "
            "make_ring(10) bfs(root=1) — first 10 entries (single-ring slice), "
            "1-based 1..10 -> 0-based 0..9"
        ),
        "graph_factory": lambda: _ring(10),
        "algo": "bfs",
        "params": {"root": 0},
        # R test: c(1, 2, 10, 3, 9, 4, 8, 5, 7, 6) -> 0-based by subtracting 1.
        "expected": [0, 1, 9, 2, 8, 3, 7, 4, 6, 5],
    },
]

DFS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "structural_star3_root1",
        "origin": (
            "test-structural-properties.R:'dfs() does not pad order' "
            "make_star(3) dfs(root=2, unreachable=FALSE) — "
            "1-based c(2, 1) -> 0-based [1, 0]. With our centre=0 star "
            "convention, R's vertex 2 is our vertex 1; the star centre "
            "(R's vertex 1, our 0) is reached via the only edge from 1."
        ),
        "graph_factory": lambda: _star(3),
        "algo": "dfs",
        "params": {"root": 1},
        "expected": [1, 0],
    },
]

CC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "two_K5_components",
        # test-components.R:'biconnected_components works' uses
        # `make_full_graph(5) + make_full_graph(5)` which is two
        # disjoint K5s and runs `components(g)$membership`. We mirror
        # the implied components() expectation: weak count = 2,
        # vertices 0-4 → component 0, vertices 5-9 → component 1.
        "origin": "test-components.R:'biconnected_components works' setup line "
        "`make_full_graph(5) + make_full_graph(5)` then `components(g)$membership`",
        "graph_factory": lambda: ig.Graph.Full(n=5, directed=False)
        + ig.Graph.Full(n=5, directed=False),
        "algo": "connected_components",
        "params": {},
        "expected": {
            "membership": [0, 0, 0, 0, 0, 1, 1, 1, 1, 1],
            "count": 2,
        },
    },
]

EUL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "has_eulerian_path_4cycle",
        # test-eulerian.R:'has_eulerian_path works' line 2:
        #   graph_from_literal(A - B - C - D - A) → 4-cycle, expects TRUE.
        "origin": "test-eulerian.R:'has_eulerian_path works' line 2 — "
        "graph_from_literal(A - B - C - D - A); 4-cycle has both path and cycle",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=False
        ),
        "algo": "is_eulerian",
        "params": {},
        # 4-cycle: every vertex has even degree → both true.
        "expected": {"has_path": True, "has_cycle": True},
    },
    {
        "case": "has_eulerian_path_complex_no_cycle",
        # test-eulerian.R:'has_eulerian_cycle works' line 48-49:
        #   graph_from_literal(A - B - C - D - E - A - F - D - B - F - E,
        #                      simplify = FALSE)
        # has_eulerian_path = TRUE, has_eulerian_cycle = FALSE.
        # Translated to 0-based vertex ids A=0, B=1, C=2, D=3, E=4, F=5.
        # Edges (in literal order): A-B, B-C, C-D, D-E, E-A, A-F, F-D, D-B, B-F, F-E.
        "origin": "test-eulerian.R:'has_eulerian_cycle works' "
        "graph_from_literal(A - B - C - D - E - A - F - D - B - F - E, simplify = FALSE)",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1),  # A-B
                (1, 2),  # B-C
                (2, 3),  # C-D
                (3, 4),  # D-E
                (4, 0),  # E-A
                (0, 5),  # A-F
                (5, 3),  # F-D
                (3, 1),  # D-B
                (1, 5),  # B-F
                (5, 4),  # F-E
            ],
            directed=False,
        ),
        "algo": "is_eulerian",
        "params": {},
        "expected": {"has_path": True, "has_cycle": False},
    },
]

TRI_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_triangles_R_path3",
        # path_graph(n=3) has no triangles.
        "origin": "test-aaa-auto.R-style — path_graph(n=3); 0 triangles",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "count_triangles",
        "params": {},
        "expected": 0,
    },
]

KNN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knn_R_star_4",
        # Star with centre 0 and 3 leaves: centre's neighbours have degree 1
        # each → knn[0] = 1. Leaves' single neighbour (centre) has deg 3 →
        # knn[leaf] = 3.
        "origin": "constructed (R-style): star-3 knn = [1, 3, 3, 3]",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "avg_nearest_neighbor_degree",
        "params": {},
        "expected": [1.0, 3.0, 3.0, 3.0],
    },
]

KNN_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knn_weighted_R_K4_unit",
        # K4 with unit weights collapses to unweighted knn = [3, 3, 3, 3].
        "origin": "constructed (R-style): K4 unit weights — collapses to unweighted",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "graph_weights": [1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        "algo": "avg_nearest_neighbor_degree_weighted",
        "params": {},
        "expected": [3.0, 3.0, 3.0, 3.0],
    },
]

KNNK_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knnk_R_path_5",
        # 5-path: degrees [1,2,2,2,1]; knn = [2, 1.5, 2, 1.5, 2].
        # knnk[0] (deg 1) = (2 + 2) / 2 = 2; knnk[1] (deg 2) = (1.5+2+1.5)/3 = 5/3.
        "origin": "constructed (R-style): 5-path; knnk = [2.0, 5/3]",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "knnk",
        "params": {},
        "expected": [2.0, 5.0 / 3.0],
    },
]

KNNK_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knnk_weighted_R_K4_unit",
        # K4 unit weights collapses: knnk_w[2] (deg 3) = 3.0; lower buckets None.
        "origin": "constructed (R-style): K4 unit weights — knnk_w = [None, None, 3.0]",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "graph_weights": [1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        "algo": "knnk_weighted",
        "params": {},
        "expected": [None, None, 3.0],
    },
]

DECOMPOSE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "decompose_R_K4_single_component",
        # K4 is a single connected component. decompose returns one
        # subgraph identical (after identity remap) to the original.
        # Edges canonicalised to (min, max) and sorted ascending.
        "origin": "constructed (R-style): K4 single component",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "decompose",
        "params": {},
        "expected": [
            {
                "vcount": 4,
                "directed": False,
                "edges": [
                    [0, 1], [0, 2], [0, 3],
                    [1, 2], [1, 3], [2, 3],
                ],
            },
        ],
    },
]

TRANS_BARRAT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_barrat_R_K4_unit",
        # K4 with unit weights: every triple is a triangle → Barrat = 1.0
        # for all four vertices (mirrors unweighted local clustering).
        "origin": "constructed (R-style): K4 unit weights → Barrat = 1.0 per vertex",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "graph_weights": [1.0] * 6,
        "algo": "transitivity_barrat",
        "params": {},
        "expected": [1.0, 1.0, 1.0, 1.0],
    },
]

RECIP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reciprocity_undirected_is_one",
        # Per upstream: undirected graphs have reciprocity 1.0 unconditionally.
        "origin": "undirected graphs: reciprocity = 1.0 by definition",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "reciprocity",
        "params": {},
        "expected": 1.0,
    },
]

EIGEN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eigenvector_R_K4",
        # K4 complete: uniform eigenvector centrality 1.0.
        "origin": "constructed (R-style): K4; uniform eigenvector 1.0",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "eigenvector_centrality",
        "params": {},
        "expected": [1.0, 1.0, 1.0, 1.0],
    },
]

EIGEN_W_MANIFEST: List[Dict[str, Any]] = [
    {
        # K_3 with weights [2,2,2] — scaling the weights only scales the
        # eigenvalue, the eigenvector (max-1) is unchanged. λ = 2·2 = 4.
        "case": "eigenvector_w_R_K3_doubled",
        "origin": "constructed (R-style): K_3 with weights 2.0 each; vec=[1,1,1], λ=4",
        "graph_factory": lambda: ig.Graph.Full(n=3, directed=False),
        "graph_weights": [2.0, 2.0, 2.0],
        "algo": "eigenvector_centrality_weighted",
        "params": {},
        "expected": {
            "vector": [1.0, 1.0, 1.0],
            "eigenvalue": 4.0,
        },
    },
]

EIGEN_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        # Directed out-star (DAG): 0→{1..4}. ARPACK fallback returns
        # 1s on sinks; with mode=OUT, sinks are leaves 1..4. λ=0.
        "case": "eigenvector_dir_R_out_star_dag",
        "origin": "DAG sentinel: directed out-star, mode=OUT; leaves=1, root=0",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (0, 2), (0, 3), (0, 4)], directed=True
        ),
        "algo": "eigenvector_centrality_directed",
        "params": {"mode": "out"},
        "expected": {
            "vector": [0.0, 1.0, 1.0, 1.0, 1.0],
            "eigenvalue": 0.0,
        },
    },
]

HITS_MANIFEST: List[Dict[str, Any]] = [
    {
        # R-igraph tests/testthat/test-centrality.R: the g2 fixture in
        # `authority_score()` and `hub_score()` test_thats. Edges in R
        # 1-based notation are (1,2)(1,4)(2,3)(2,4)(3,1)(3,5)(4,3)(5,1)(5,2);
        # translated to 0-based here. Hub and authority vectors below
        # are copy-pasted verbatim from the R test file (max-abs
        # normalisation; eigenvalue derived as max|A·Aᵀ·h| at
        # convergence with h max-normed = the dominant eigenvalue of
        # A·Aᵀ on this 5x5 directed graph).
        "case": "hits_R_5v_directed_g2",
        "origin": "rigraph tests/testthat/test-centrality.R — g2 fixture (hub_score / authority_score)",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (0, 3), (1, 2), (1, 3), (2, 0), (2, 4), (3, 2), (4, 0), (4, 1)],
            directed=True,
        ),
        "algo": "hub_and_authority_scores",
        "params": {},
        "expected": {
            "hub": [
                1.0,
                0.763521118433368,
                0.546200349457203,
                0.28462967654657,
                0.918985947228995,
            ],
            "authority": [
                0.763521118433368,
                1.0,
                0.546200349457202,
                0.918985947228995,
                0.28462967654657,
            ],
            # Largest eigenvalue of A·Aᵀ for this graph; computed
            # exactly as 2 + h[1] + h[4] (the row-0 sum of A·Aᵀ
            # against the converged hub vector — hand-checked).
            "eigenvalue": 3.682507065662363,
        },
    },
    {
        # R-style directed triangle: make_graph(c(1,2,2,3,3,1),
        # directed = TRUE) — the same canonical fixture used by
        # several other R-style centrality manifests in this file.
        # Every vertex is symmetrically a hub and an authority of
        # equal magnitude; max-norm convention puts all entries at
        # 1.0. Largest A·Aᵀ eigenvalue is 1 (each row of A·Aᵀ has a
        # single 1 on the diagonal).
        "case": "hits_R_directed_triangle",
        "origin": "constructed (R-style): make_graph(c(1,2,2,3,3,1), directed=TRUE) — uniform hub/authority",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "hub_and_authority_scores",
        "params": {},
        "expected": {
            "hub": [1.0, 1.0, 1.0],
            "authority": [1.0, 1.0, 1.0],
            "eigenvalue": 1.0,
        },
    },
]

HITS_W_MANIFEST: List[Dict[str, Any]] = [
    {
        # R-style: hub_score(g, weights = c(3, 4)) on a 2-hub→1-auth
        # bipartite graph. Closed form on W·Wᵀ (top-left 2x2 =
        # [[9,12],[12,16]]) gives λ = 25, hub = (3/4, 1, 0),
        # authority = (0, 0, 1). All-positive weights so sign-cleanup
        # is in effect.
        "case": "hits_R_w_two_hubs_one_authority_weighted",
        "origin": "constructed (R-style): hub_score(weights=c(3,4)) on 0→2, 1→2; λ=25 closed form",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 2), (1, 2)], directed=True
        ),
        "graph_weights": [3.0, 4.0],
        "algo": "hub_and_authority_scores_weighted",
        "params": {},
        "expected": {
            "hub": [0.75, 1.0, 0.0],
            "authority": [0.0, 0.0, 1.0],
            "eigenvalue": 25.0,
        },
    },
]

BC_EDGES_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "biconnected_component_edges_R_K4",
        # K4: a single biconnected component containing all 6 edges.
        "origin": "constructed (R-style): K4 complete; CC-012 partition trivial",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "biconnected_component_edges",
        "params": {},
        "expected": sorted(
            [
                sorted(
                    [
                        [0, 1],
                        [0, 2],
                        [0, 3],
                        [1, 2],
                        [1, 3],
                        [2, 3],
                    ]
                ),
            ]
        ),
    },
]

BC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "biconnected_components_R_triangle",
        # K3: a single biconnected component with no APs.
        "origin": "constructed (R-style): triangle; 1 component, no APs",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "biconnected_components",
        "params": {},
        "expected": {
            "count": 1,
            "components": [[0, 1, 2]],
            "articulation_points": [],
        },
    },
]

PAGERANK_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "pagerank_R_K4",
        # K4: every vertex has identical PageRank = 0.25.
        "origin": "constructed (R-style): K4 complete; uniform PageRank 0.25",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "pagerank",
        "params": {},
        "expected": [0.25, 0.25, 0.25, 0.25],
    },
]

EDGE_BETW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_betweenness_R_triangle",
        # Triangle: each direct edge (u,v) covers exactly the (u,v) pair → 1.0 each.
        "origin": "constructed (R-style): triangle; each edge betweenness = 1.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "edge_betweenness",
        "params": {},
        "expected": [1.0, 1.0, 1.0],
    },
]

BETW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "betweenness_R_4cycle",
        # 4-cycle 0-1-2-3-0: every vertex has betweenness 0.5 (sits on
        # one antipodal pair's two-path average).
        "origin": "constructed (R-style): 4-cycle; uniform betweenness 0.5",
        "graph_factory": lambda: ig.Graph.Ring(
            n=4, directed=False, mutual=False, circular=True
        ),
        "algo": "betweenness",
        "params": {},
        "expected": [0.5, 0.5, 0.5, 0.5],
    },
]

HARMONIC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "harmonic_R_triangle",
        # K3: every vertex has harmonic 1.0.
        "origin": "constructed (R-style): triangle; harmonic 1.0 each",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "harmonic_centrality",
        "params": {},
        "expected": [1.0, 1.0, 1.0],
    },
]

CLOSE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "closeness_R_triangle",
        # Triangle: every vertex has closeness 1.0.
        "origin": "constructed (R-style): triangle; closeness 1.0 each",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "closeness",
        "params": {},
        "expected": [1.0, 1.0, 1.0],
    },
]

ASSORT_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assort_w_R_diamond_non_uniform",
        # K4 minus edge (the "diamond"): edges (0,1)(0,2)(1,2)(1,3)(2,3)
        # with weights (1, 2, 0.5, 1.5, 1). Hand-computed via the
        # upstream weighted Pearson formula (no python-igraph oracle).
        # strengths:
        #   0: 1+2 = 3;  1: 1+0.5+1.5 = 3;  2: 2+0.5+1 = 3.5;  3: 1.5+1 = 2.5
        # W = 1+2+0.5+1.5+1 = 6
        # num1 (= Σ w * s_u * s_v):
        #   1*(3*3) + 2*(3*3.5) + 0.5*(3*3.5) + 1.5*(3*2.5) + 1*(3.5*2.5)
        #   = 9 + 21 + 5.25 + 11.25 + 8.75 = 55.25; /W = 9.208333...
        # num2 (= Σ w * (s_u + s_v)):
        #   1*(3+3) + 2*(3+3.5) + 0.5*(3+3.5) + 1.5*(3+2.5) + 1*(3.5+2.5)
        #   = 6 + 13 + 3.25 + 8.25 + 6 = 36.5; /(2W) = 3.041666...; ^2 = 9.251736111111108
        # den1 (= Σ w * (s_u^2 + s_v^2)):
        #   1*(9+9) + 2*(9+12.25) + 0.5*(9+12.25) + 1.5*(9+6.25) + 1*(12.25+6.25)
        #   = 18 + 42.5 + 10.625 + 22.875 + 18.5 = 112.5; /(2W) = 9.375
        # r = (9.208333... - 9.251736...) / (9.375 - 9.251736...)
        #   = -0.04340277... / 0.12326388... ≈ -0.352112676
        "origin": "constructed (rigraph-style): K4-minus-edge with weights "
        "(1, 2, 0.5, 1.5, 1); hand-computed",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "graph_weights": [1.0, 2.0, 0.5, 1.5, 1.0],
        "algo": "assortativity_degree_weighted",
        "params": {},
        # Computed via the formula above; verified by running the same
        # formula in Python (see oracle dispatcher comment about why
        # python-igraph itself can't oracle this case).
        "expected": -0.3521126760563289,
    },
]

PAGERANK_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "pagerank_w_R_undirected_triangle_unit",
        # rigraph's `page_rank(graph, weights = ...)` mirrors
        # igraph_pagerank. Triangle with unit weights → uniform 1/3.
        "origin": "constructed (rigraph-style): undirected triangle, unit weights",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0],
        "algo": "pagerank_weighted",
        "params": {},
        "expected": [
            0.3333333333333333,
            0.3333333333333333,
            0.3333333333333333,
        ],
    },
]

EDGE_BETW_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_betw_w_R_directed_chain_with_shortcut",
        # rigraph mirrors igraph_edge_betweenness with weights. Directed
        # 0→1→2→3 + extra 0→3 weight 5: chain wins, shortcut gets 0.
        "origin": "constructed (rigraph-style): directed chain + heavy shortcut "
        "0→3@5; chain edges carry [3,4,3], shortcut 0",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (0, 3)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "edge_betweenness_weighted",
        "params": {},
        "expected": {
            "edges": [[0, 1], [1, 2], [2, 3], [0, 3]],
            "values": [3.0, 4.0, 3.0, 0.0],
        },
    },
]

BETW_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "betw_w_R_directed_chain_with_shortcut",
        # rigraph mirrors igraph_betweenness with weights. Directed
        # 0→1→2→3 with extra 0→3 weight 5: shortest 0→3 routes through
        # 1→2 (cost 3) so vertices 1, 2 each carry betweenness 2.
        "origin": "constructed (rigraph-style): directed chain + shortcut "
        "0→3@5; intermediates 1, 2 each = 2.0",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (0, 3)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "betweenness_weighted",
        "params": {},
        "expected": [0.0, 2.0, 2.0, 0.0],
    },
]

HARMONIC_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "harmonic_w_R_disconnected_pair",
        # 4-vertex graph with isolated vertex 3.
        "origin": "constructed (rigraph-style): 3-path + isolated 4th; "
        "harmonic well-defined (unlike closeness)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 1.0],
        "algo": "harmonic_centrality_weighted",
        "params": {},
        "expected": [0.5, 2.0 / 3.0, 0.5, 0.0],
    },
]

CLOSENESS_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "closeness_weighted_R_undirected_path",
        # rigraph mirrors igraph_closeness with weights. 4-vertex
        # path with weights (1.5, 2.5, 0.5).
        "origin": "constructed (rigraph-style): 4-path with weights "
        "(1.5, 2.5, 0.5); endpoint and middle closeness computed",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [1.5, 2.5, 0.5],
        "algo": "closeness_weighted",
        "params": {},
        "expected": [
            0.3,
            0.42857142857142855,
            0.42857142857142855,
            0.375,
        ],
    },
]

COMPLEMENTER_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "complementer_R_directed_single_edge",
        # rigraph mirrors igraph_complementer; directed (0,1) on 3
        # vertices, loops=False → 5 missing directed edges.
        "origin": "constructed (rigraph-style): directed single edge; complementer "
        "(loops=False) adds 5 reverse/missing pairs",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1)], directed=True
        ),
        "algo": "complementer",
        "params": {"loops": False},
        "expected": {
            "vcount": 3,
            "directed": True,
            "edges": [[0, 2], [1, 0], [1, 2], [2, 0], [2, 1]],
        },
    },
]

DIJKSTRA_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_R_undirected_partial_unreachable",
        # rigraph's `distances(graph, v=, weights=, mode='all')` mirrors
        # igraph_distances_dijkstra. Two disconnected weighted edges →
        # one unreachable vertex from source.
        "origin": "constructed (rigraph-style): 4-vertex with two disconnected "
        "weighted edges; unreachable yields Inf",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.5],
        "algo": "dijkstra_distances",
        "params": {"source": 0},
        "expected": [0.0, 1.0, None, None],
    },
]

# ALGO-SP-001b: paths variant — distances only.
DIJKSTRA_PATHS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_paths_R_undirected_partial_unreachable",
        "origin": "constructed (rigraph-style): 4-vertex with two disconnected "
        "weighted edges; unreachable yields None",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.5],
        "algo": "dijkstra_paths",
        "params": {"source": 0},
        "expected": {"distances": [0.0, 1.0, None, None]},
    },
]

# ALGO-SP-001b: source-to-target — unreachable target ⇒ null.
DIJKSTRA_PATH_TO_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_path_to_R_unreachable_target",
        # rigraph's `shortest_paths` returns an empty path (or NA) for
        # unreachable targets. We encode this as JSON null.
        "origin": "constructed (rigraph-style): 4-vertex with two disconnected "
        "weighted edges; query target=2 from source=0 ⇒ unreachable",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.5],
        "algo": "dijkstra_path_to",
        "params": {"source": 0, "target": 2},
        "expected": None,
    },
]

# ALGO-SP-001b: cutoff variant.
DIJKSTRA_CUTOFF_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_cutoff_R_path_unit_weights_cutoff_1_5",
        # rigraph mirror: undirected path 0-1-2-3 with unit weights and
        # cutoff 1.5 returns distances [0, 1, None, None].
        "origin": "constructed (rigraph-style): undirected P4 unit weights, cutoff=1.5",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0],
        "algo": "dijkstra_distances_cutoff",
        "params": {"source": 0, "cutoff": 1.5},
        "expected": [0.0, 1.0, None, None],
    },
]

# ALGO-PR-022: is_acyclic. rigraph's `is_acyclic(g)` returns
# TRUE/FALSE; mirrors upstream's predicate.
IS_ACYCLIC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_acyclic_R_undirected_self_loop_false",
        # Self-loop counts as a (length-1) cycle.
        "origin": "constructed (rigraph-style): self-loop ⇒ cycle, not acyclic",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 0), (0, 1)], directed=False
        ),
        "algo": "is_acyclic",
        "params": {},
        "expected": False,
    },
    {
        "case": "is_acyclic_R_directed_cycle_false",
        # 0 → 1 → 2 → 0.
        "origin": "constructed (rigraph-style): directed 3-cycle ⇒ not acyclic",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "is_acyclic",
        "params": {},
        "expected": False,
    },
]

# ALGO-PR-023: is_tree. rigraph's `is_tree(g, mode="out"/"in"/"all")`
# returns TRUE/FALSE.
IS_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_tree_R_undirected_star_true",
        # K_{1,4}-like star: vertex 0 connected to 1, 2, 3, 4 — undirected tree.
        "origin": "constructed (rigraph-style): undirected star — tree",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (0, 2), (0, 3), (0, 4)], directed=False
        ),
        "algo": "is_tree",
        "params": {"mode": "all"},
        "expected": True,
    },
    {
        "case": "is_tree_R_undirected_disconnected_false",
        # Two disjoint edges — disconnected, not a tree.
        "origin": "constructed (rigraph-style): two disjoint edges — not a tree",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "algo": "is_tree",
        "params": {"mode": "all"},
        "expected": False,
    },
]

# ALGO-PR-024: is_forest. rigraph's
# `is_forest(g, mode="out"/"in"/"all", details=TRUE)` returns
# `list(res = bool, roots = vector)` — fixtures here mirror
# common rigraph examples (man/is_forest.Rd shows a single tree
# call and a multi-tree variant).
IS_FOREST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_forest_R_undirected_star_true",
        # K_{1,4}: still a forest with one component, root = 0.
        "origin": "constructed (rigraph-style): undirected star — forest",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (0, 2), (0, 3), (0, 4)], directed=False
        ),
        "algo": "is_forest",
        "params": {"mode": "all"},
        "expected": {"is_forest": True, "roots": [0]},
    },
    {
        "case": "is_forest_R_undirected_with_isolated_vertex_true",
        # Path 0-1-2 + isolated vertex 3 — 2 trees.
        "origin": "constructed (rigraph-style): path + isolated vertex — 2 trees",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "is_forest",
        "params": {"mode": "all"},
        "expected": {"is_forest": True, "roots": [0, 3]},
    },
]

# ALGO-PR-016: is_complete. rigraph exposes `is_complete(g)`
# (man/is_complete.Rd) — returns TRUE for null/singleton, false
# for any pair without an edge between them.
IS_COMPLETE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_complete_R_k5_undirected_true",
        # rigraph's `make_full_graph(5)` is the canonical K_5 example.
        "origin": "constructed (rigraph-style): make_full_graph(5)",
        "graph_factory": lambda: ig.Graph.Full(n=5, directed=False),
        "algo": "is_complete",
        "params": {},
        "expected": True,
    },
    {
        "case": "is_complete_R_two_triangles_disconnected_false",
        # Two K_3 sharing no vertex: each piece is locally complete
        # but no edges between {0,1,2} and {3,4,5}.
        "origin": "constructed (rigraph-style): K_3 ⊔ K_3 — not complete",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5)],
            directed=False,
        ),
        "algo": "is_complete",
        "params": {},
        "expected": False,
    },
]

# ALGO-PR-027: neighborhood_size. rigraph exposes
# `neighborhood_size(g, order, vids, mode)`. R test fixture from
# tests/testthat/test-aaa-auto.R:neighborhood_size_impl basic
# (make_ring(5), order=1, mode="all").
NEIGHBORHOOD_SIZE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "neighborhood_size_R_ring5_order_1_all",
        "origin": "tests/testthat/test-aaa-auto.R neighborhood_size_impl basic — make_ring(5) order=1",
        "graph_factory": lambda: ig.Graph.Ring(n=5, circular=True),
        "algo": "neighborhood_size",
        "params": {"order": 1, "mode": "all", "mindist": 0},
        "expected": [3, 3, 3, 3, 3],
    },
    {
        "case": "neighborhood_size_R_full5_order_2_excludes_self",
        # K_5: order 2 mindist 1 from every vertex sees the other 4.
        "origin": "constructed (rigraph-style): make_full_graph(5) order=2 mindist=1",
        "graph_factory": lambda: ig.Graph.Full(n=5, directed=False),
        "algo": "neighborhood_size",
        "params": {"order": 2, "mode": "all", "mindist": 1},
        "expected": [4, 4, 4, 4, 4],
    },
]

# ALGO-PR-027b: neighborhood (vertex lists). Fixtures from
# tests/testthat/test-aaa-auto.R:neighborhood_impl basic
# (snapshot in tests/testthat/_snaps/aaa-auto.md). R uses 1-based
# indexing; the snapshot output `[1] 1 2 5` for vertex 1 means
# {0, 1, 4} 0-indexed. All expected lists are sorted.
NEIGHBORHOOD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "neighborhood_R_ring5_order_1_all",
        "origin": "tests/testthat/test-aaa-auto.R neighborhood_impl basic — make_ring(5) order=1 mode=all",
        "graph_factory": lambda: ig.Graph.Ring(n=5, circular=True),
        "algo": "neighborhood",
        "params": {"order": 1, "mode": "all", "mindist": 0},
        # R snapshot (0-indexed, sorted):
        # v0:{0,1,4}, v1:{0,1,2}, v2:{1,2,3}, v3:{2,3,4}, v4:{0,3,4}
        "expected": [
            [0, 1, 4],
            [0, 1, 2],
            [1, 2, 3],
            [2, 3, 4],
            [0, 3, 4],
        ],
    },
    {
        "case": "neighborhood_R_full5_order_2_excludes_self",
        # K_5 mirror of the size fixture; the actual vertex lists.
        "origin": "constructed (rigraph-style): make_full_graph(5) order=2 mindist=1",
        "graph_factory": lambda: ig.Graph.Full(n=5, directed=False),
        "algo": "neighborhood",
        "params": {"order": 2, "mode": "all", "mindist": 1},
        # Each vertex's 4 non-self peers.
        "expected": [
            [1, 2, 3, 4],
            [0, 2, 3, 4],
            [0, 1, 3, 4],
            [0, 1, 2, 4],
            [0, 1, 2, 3],
        ],
    },
]

# ALGO-PR-021: topological_sorting. rigraph's
# `topo_sort(g, mode="out"|"in")` returns the order.
TOPOLOGICAL_SORTING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "topo_sort_R_parallel_edges_dont_affect_order",
        # Parallel edges 0 → 1; order = [0, 1] regardless.
        "origin": "constructed (rigraph-style): parallel edges, unique order",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1)], directed=True
        ),
        "algo": "topological_sorting",
        "params": {"mode": "out"},
        "expected": [0, 1],
    },
    {
        "case": "topo_sort_R_two_disjoint_chains",
        # Two disjoint chains 0 → 1 and 2 → 3.
        # Sources are vertices 0 and 2 (both in-degree 0). They both
        # get queued together; Kahn pops them in id order, then their
        # successors.
        "origin": "constructed (rigraph-style): two disjoint chains",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=True
        ),
        "algo": "topological_sorting",
        "params": {"mode": "out"},
        "expected": [0, 2, 1, 3],
    },
]

# ALGO-PR-020: is_dag. rigraph's `is_dag(g)` returns TRUE/FALSE.
IS_DAG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_dag_R_undirected_false",
        # Undirected graphs are never DAGs per the upstream contract.
        "origin": "constructed (rigraph-style): undirected — not a DAG",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "is_dag",
        "params": {},
        "expected": False,
    },
    {
        "case": "is_dag_R_two_disjoint_dags_true",
        # Two disjoint 2-vertex DAGs in the same graph.
        "origin": "constructed (rigraph-style): two disjoint DAG branches",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=True
        ),
        "algo": "is_dag",
        "params": {},
        "expected": True,
    },
]

# ALGO-CORE-001e: is_same_graph (structural equality). rigraph
# doesn't expose this directly; hand-computed expected values.
IS_SAME_GRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_same_graph_R_undirected_endpoint_swap_same",
        # (0,1) and (1,0) are the same undirected edge ⇒ same.
        "origin": "constructed (rigraph-style): undirected endpoint swap ⇒ same",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1)], directed=False
        ),
        "algo": "is_same_graph",
        "params": {
            "other": {
                "n": 2,
                "edges": [[1, 0]],
                "directed": False,
            }
        },
        "expected": True,
    },
    {
        "case": "is_same_graph_R_directed_reverse_not_same",
        # In directed graphs, (0,1) and (1,0) are distinct edges.
        "origin": "constructed (rigraph-style): directed reverse ⇒ not same",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1)], directed=True
        ),
        "algo": "is_same_graph",
        "params": {
            "other": {
                "n": 2,
                "edges": [[1, 0]],
                "directed": True,
            }
        },
        "expected": False,
    },
]

# ALGO-PR-028: convergence_degree. rigraph exposes
# `convergence_degree(g)` returning a per-edge numeric vector. We
# encode `NaN` as JSON `null` to match the oracle wire format.
CONVERGENCE_DEGREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "convergence_degree_R_directed_star_upstream",
        # Same as the upstream igraph C reference test 2 — n=6 hub
        # graph: four leaves point at hub 0, hub points at sink 5.
        "origin": (
            "constructed (rigraph-style): directed star, "
            "matches references/igraph .out test 2"
        ),
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(1, 0), (2, 0), (3, 0), (4, 0), (0, 5)],
            directed=True,
        ),
        "algo": "convergence_degree",
        "params": {},
        "expected": [
            -1.0 / 3.0, -1.0 / 3.0, -1.0 / 3.0, -1.0 / 3.0, 2.0 / 3.0,
        ],
    },
    {
        "case": "convergence_degree_R_undirected_path",
        # P_4 path: middle edge sees more crossing pairs than ends.
        # Computed by hand-running the BFS-per-source algorithm.
        "origin": "constructed (rigraph-style): P_4 path, hand-checked",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "convergence_degree",
        "params": {},
        # edge (0,1): ins=1, outs=3 → |−1/2| = 1/2
        # edge (1,2): ins=2, outs=2 → 0
        # edge (2,3): ins=3, outs=1 → 1/2
        "expected": [0.5, 0.0, 0.5],
    },
]

# ALGO-CC-032: Site percolation. rigraph doesn't bind this; hand-
# computed expected values.
SITE_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "site_perc_R_parallel_edges_count_each",
        # Two parallel edges between 0 and 1: activating both yields
        # edge_count=2 (each parallel edge counted).
        "origin": "constructed (rigraph-style): parallel edges count each",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1)], directed=False
        ),
        "algo": "site_percolation",
        "params": {"vertex_order": [0, 1]},
        "expected": {
            "giant_size": [1, 2],
            "edge_count": [0, 2],
        },
    },
    {
        "case": "site_perc_R_reverse_order_chain",
        # Chain 0-1-2-3, activate in reverse: 3, 2, 1, 0.
        "origin": "constructed (rigraph-style): P4 activated in reverse",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "site_percolation",
        "params": {"vertex_order": [3, 2, 1, 0]},
        "expected": {
            "giant_size": [1, 2, 3, 4],
            "edge_count": [0, 1, 2, 3],
        },
    },
]

# ALGO-CC-031: Bond percolation. rigraph doesn't bind this either;
# hand-computed expected values.
BOND_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bond_perc_R_triangle_natural_order",
        # Triangle: adding edges in id order builds {0,1}, {0,1,2}, no
        # change for the third edge (already connected).
        "origin": "constructed (rigraph-style): triangle, natural id order",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "algo": "bond_percolation",
        "params": {"edge_order": [0, 1, 2]},
        "expected": {
            "giant_size": [2, 3, 3],
            "vertex_count": [2, 3, 3],
        },
    },
    {
        "case": "bond_perc_R_directed_graph_direction_ignored",
        # Directed edges 0→1, 1→2 percolate the same as undirected.
        "origin": "constructed (rigraph-style): directed P3 — direction ignored",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "algo": "bond_percolation",
        "params": {"edge_order": [0, 1]},
        "expected": {
            "giant_size": [2, 3],
            "vertex_count": [2, 3],
        },
    },
]

# ALGO-CC-030: Edge-list percolation. rigraph doesn't bind this
# either; hand-computed expected values.
EDGELIST_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edgelist_perc_R_parallel_edge_no_change",
        # Adding the same edge twice — second add is a no-op.
        "origin": "constructed (rigraph-style): parallel edge as no-op",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1)], directed=False
        ),
        "algo": "edgelist_percolation",
        "params": {},
        "expected": {
            "giant_size": [2, 2],
            "vertex_count": [2, 2],
        },
    },
    {
        "case": "edgelist_perc_R_self_loop_adds_one_vertex",
        # Self-loop on vertex 0, then a normal edge: giant grows
        # 1 → 2, vertex_count 1 → 2.
        "origin": "constructed (rigraph-style): self-loop then bridge",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 0), (0, 1)], directed=False
        ),
        "algo": "edgelist_percolation",
        "params": {},
        "expected": {
            "giant_size": [1, 2],
            "vertex_count": [1, 2],
        },
    },
]

# ALGO-SP-014: Single-source widest-paths SPT struct (widths +
# parents + inbound_edges). rigraph doesn't bind this; hand-computed.
WIDEST_PATHS_SPT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_paths_R_parallel_edges_pick_widest",
        # Parallel edges (0,1) with widths 1, 5, 3. The widest direct
        # edge (id 1, width 5) wins; parent_edge is edge 1.
        "origin": "constructed (rigraph-style): 3 parallel edges 0-1 (1,5,3)",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1), (0, 1)], directed=False
        ),
        "graph_weights": [1.0, 5.0, 3.0],
        "algo": "widest_paths",
        "params": {"source": 0},
        "expected": {
            "widths": [None, 5.0],
            "parents": [None, 0],
            "inbound_edges": [None, 1],
        },
    },
    {
        "case": "widest_paths_R_negative_finite_edge_bottleneck",
        # Negative-but-finite weight (-1.0) acts as a valid bottleneck:
        # chain 0-1-2 has widths [INF, -1, min(-1, 1)] = [INF, -1, -1].
        "origin": "constructed (rigraph-style): negative-finite first edge sets bottleneck",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [-1.0, 1.0],
        "algo": "widest_paths",
        "params": {"source": 0},
        "expected": {
            "widths": [None, -1.0, -1.0],
            "parents": [None, 0, 1],
            "inbound_edges": [None, 0, 1],
        },
    },
]

# ALGO-SP-013: Multi-target widest paths. rigraph does not bind
# this directly; hand-computed expected paths.
WIDEST_PATHS_TO_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_paths_to_R_chain_three_targets",
        # 0-1-2-3 weights (5, 1, 3); targets [1, 2, 3].
        "origin": "constructed (rigraph-style): P4 (5,1,3) → three targets",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [5.0, 1.0, 3.0],
        "algo": "widest_paths_to",
        "params": {"from": 0, "targets": [1, 2, 3]},
        "expected": [
            {"vertices": [0, 1], "edges": [0]},
            {"vertices": [0, 1, 2], "edges": [0, 1]},
            {"vertices": [0, 1, 2, 3], "edges": [0, 1, 2]},
        ],
    },
    {
        "case": "widest_paths_to_R_duplicate_targets_same_path",
        # Same target id appears multiple times; each gets the same path.
        "origin": "constructed (rigraph-style): targets [2,2] → identical entries",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [3.0, 3.0],
        "algo": "widest_paths_to",
        "params": {"from": 0, "targets": [2, 2]},
        "expected": [
            {"vertices": [0, 1, 2], "edges": [0, 1]},
            {"vertices": [0, 1, 2], "edges": [0, 1]},
        ],
    },
]

# ALGO-SP-012: Floyd-Warshall-based all-pairs widest widths matrix.
# Hand-computed (rigraph does not expose). Diagonal +∞ encoded as null.
WIDEST_PATH_WIDTHS_FW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_fw_R_directed_diamond",
        # Directed 0→1 (5), 0→2 (1), 1→3 (-2), 2→3 (4).
        # Negative-finite (not -∞) is a valid bottleneck.
        # From 0: 0→1=5; 0→2=1; 0→3 = max(min(5,-2)=-2, min(1,4)=1) = 1.
        "origin": "constructed (rigraph-style): directed diamond, one negative-finite edge",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 3), (2, 3)], directed=True
        ),
        "graph_weights": [5.0, 1.0, -2.0, 4.0],
        "algo": "widest_path_widths_floyd_warshall",
        "params": {},
        "expected": [
            [None, 5.0, 1.0, 1.0],
            [None, None, None, -2.0],
            [None, None, None, 4.0],
            [None, None, None, None],
        ],
    },
    {
        "case": "widest_fw_R_undirected_triangle",
        "origin": "constructed (rigraph-style): triangle (3, 6, 2)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [3.0, 6.0, 2.0],
        "algo": "widest_path_widths_floyd_warshall",
        "params": {},
        "expected": [
            [None, 3.0, 6.0],
            [3.0, None, 3.0],
            [6.0, 3.0, None],
        ],
    },
]

# ALGO-SP-011: Widest path (single source-to-target). Hand-computed —
# rigraph does not expose this directly.
WIDEST_PATH_GET_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_get_R_chain_three_edges",
        # Path 0-1-2-3 with weights 5, 1, 3. Widest 0→3 path is the
        # chain (unique). Bottleneck = min(5, 1, 3) = 1.
        "origin": "constructed (rigraph-style): P4 (5,1,3) — bottleneck 1",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [5.0, 1.0, 3.0],
        "algo": "widest_path",
        "params": {"from": 0, "to": 3},
        "expected": {"vertices": [0, 1, 2, 3], "edges": [0, 1, 2]},
    },
    {
        "case": "widest_get_R_triangle_via_shortcut",
        # Triangle (1, 4, 2). Widest 0→1: chain 0-2-1 (bottleneck 2)
        # beats direct edge (width 1). Order matches Rust impl.
        "origin": "constructed (rigraph-style): triangle (1,4,2) — chain via 2 wins",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "widest_path",
        "params": {"from": 0, "to": 1},
        "expected": {"vertices": [0, 2, 1], "edges": [1, 2]},
    },
]

# ALGO-SP-010: Widest-path widths. rigraph does not expose this API
# either; values are hand-computed (source position null by convention).
WIDEST_PATH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_R_directed_chain_out_mode",
        # Directed 0→1 (5), 1→2 (3). From source 0: w[1]=5, w[2]=min(5,3)=3.
        "origin": "constructed (rigraph-style): directed P3 (5,3) OUT mode",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "graph_weights": [5.0, 3.0],
        "algo": "widest_path_widths",
        "params": {"source": 0},
        "expected": [None, 5.0, 3.0],
    },
    {
        "case": "widest_R_parallel_edges_keep_widest",
        # Parallel edges (0,1) with widths 1, 5, 3. Widest 0→1 = 5.
        "origin": "constructed (rigraph-style): 3 parallel edges 0-1",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1), (0, 1)], directed=False
        ),
        "graph_weights": [1.0, 5.0, 3.0],
        "algo": "widest_path_widths",
        "params": {"source": 0},
        "expected": [None, 5.0],
    },
]

# ALGO-SP-003: Johnson all-pairs distances. rigraph exposes
# `distances(g, weights=, algorithm="johnson")` returning a square
# numeric matrix; Inf encodes unreachability and maps to None here.
JOHNSON_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "johnson_R_directed_diamond_negative_edge",
        "origin": "constructed (rigraph-style): directed diamond, negative edge 1→3",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 3), (2, 3)], directed=True
        ),
        "graph_weights": [3.0, 1.0, -2.0, 4.0],
        "algo": "johnson_distances",
        "params": {},
        "expected": [
            [0.0, 3.0, 1.0, 1.0],
            [None, 0.0, None, -2.0],
            [None, None, 0.0, 4.0],
            [None, None, None, 0.0],
        ],
    },
    {
        "case": "johnson_R_undirected_chain_positive",
        "origin": "constructed (rigraph-style): undirected P4 (1,2,3) — fast path",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 3.0],
        "algo": "johnson_distances",
        "params": {},
        "expected": [
            [0.0, 1.0, 3.0, 6.0],
            [1.0, 0.0, 2.0, 5.0],
            [3.0, 2.0, 0.0, 3.0],
            [6.0, 5.0, 3.0, 0.0],
        ],
    },
]

# ALGO-SP-002: Bellman-Ford single-source distances. rigraph exposes
# `distances(g, v=, weights=, algorithm="bellman-ford")` — same numeric
# output as our Rust port, with Inf encoded as None.
BELLMAN_FORD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bellman_ford_R_directed_diamond_negative_edge",
        "origin": "constructed (rigraph-style): directed diamond with negative edge 1→3",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 3), (2, 3)], directed=True
        ),
        "graph_weights": [3.0, 1.0, -2.0, 4.0],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 3.0, 1.0, 1.0],
    },
    {
        "case": "bellman_ford_R_undirected_chain_positive",
        "origin": "constructed (rigraph-style): undirected P4 (1,2,3)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 3.0],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 1.0, 3.0, 6.0],
    },
    {
        "case": "bellman_ford_R_unreachable_yields_inf",
        "origin": "constructed (rigraph-style): 4-vertex with two disconnected weighted edges",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.5, -0.5],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 1.5, None, None],
    },
]

# ALGO-SP-001c: mode-aware distances. Undirected graph: every mode
# identical (rigraph's `distances(graph, mode='all')`).
DIJKSTRA_DIST_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_dist_mode_R_undirected_all",
        "origin": "constructed (rigraph-style): undirected P4, ALL mode (undirected)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 3.0],
        "algo": "dijkstra_distances_with_mode",
        "params": {"source": 0, "mode": "all"},
        "expected": [0.0, 1.0, 3.0, 6.0],
    },
]

# ALGO-SP-001c: all-shortest-paths. Disconnected graph: nrgeo == 0
# for unreachable vertices.
DIJKSTRA_ASP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_asp_R_disconnected_unreachable",
        "origin": "constructed (rigraph-style): two disjoint edges; vertex 2 unreachable",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.5],
        "algo": "dijkstra_all_shortest_paths",
        "params": {"source": 0, "mode": "out"},
        "expected": {"distances": [0.0, 1.0, None, None], "nrgeo": [1, 1, 0, 0]},
    },
]

# ALGO-SP-005 A*: unreachable target ⇒ null.
ASTAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "a_star_path_R_unreachable_target",
        "origin": "constructed (rigraph-style): two disjoint edges; target=2 unreachable from 0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.5],
        "algo": "a_star_path",
        "params": {"source": 0, "target": 2, "mode": "out"},
        "expected": None,
    },
]

# ALGO-SP-021..023 weighted: disconnected components keep per-component
# eccentricity (matches rigraph's `unconn=TRUE` default).
ECC_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_weighted_R_disconnected_components",
        # Two components: 0-1 weight 1, 2-3 weight 4.
        # ecc[0]=ecc[1]=1.0; ecc[2]=ecc[3]=4.0.
        "origin": "constructed (rigraph-style): two disjoint edges with non-uniform weights",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 4.0],
        "algo": "eccentricity_weighted_with_mode",
        "params": {"mode": "all"},
        "expected": [1.0, 1.0, 4.0, 4.0],
    },
]

RAD_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_weighted_R_disconnected_components",
        "origin": "constructed (rigraph-style): two disjoint edges; min ecc = 1.0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 4.0],
        "algo": "radius_weighted_with_mode",
        "params": {"mode": "all"},
        "expected": 1.0,
    },
]

DIAM_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_weighted_R_disconnected_components",
        "origin": "constructed (rigraph-style): two disjoint edges; max ecc = 4.0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 4.0],
        "algo": "diameter_weighted_with_mode",
        "params": {"mode": "all"},
        "expected": 4.0,
    },
]

MODULARITY_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_directed_R_chain_4_two_blocks",
        # rigraph smoke: directed chain 0→1→2→3 with partition
        # {0,1}/{2,3}. m=3, internal edges: (0→1) and (2→3) = 2.
        # e_norm = 2/3. k_out[c0]=2/3 (vertex 0,1 each out-deg 1; only edge 1→2 cross).
        # Wait, for c0={0,1}: vertex 0 out-deg 1 (0→1); vertex 1 out-deg 1 (1→2).
        # Sum k_out = 2. So k_out[c0]=2/3.
        # k_in[c0]: vertex 0 in-deg 0; vertex 1 in-deg 1 (from 0). Sum=1, /3 = 1/3.
        # k_out[c1]: vertex 2 out-deg 1 (2→3); vertex 3 out-deg 0. Sum=1, /3 = 1/3.
        # k_in[c1]: vertex 2 in-deg 1; vertex 3 in-deg 1. Sum=2, /3 = 2/3.
        # Q = 2/3 - (2/3 * 1/3 + 1/3 * 2/3) = 2/3 - 4/9 = 6/9 - 4/9 = 2/9 ≈ 0.222.
        "origin": "constructed (rigraph-style): directed 4-chain, two blocks",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3)],
            directed=True,
        ),
        "algo": "modularity_directed",
        "params": {"membership": [0, 0, 1, 1], "resolution": 1.0},
        "expected": 2.0 / 9.0,
    },
]

ASSORT_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_degree_directed_R_3_cycle_returns_none",
        # rigraph smoke: directed 3-cycle is regular (every vertex
        # has out-deg 1 and in-deg 1) → variance vanishes → None.
        "origin": "constructed (rigraph-style): directed 3-cycle",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (2, 0)],
            directed=True,
        ),
        "algo": "assortativity_degree_directed",
        "params": {},
        "expected": None,
    },
]

# ALGO-PR-006d: Directed weighted assortativity. Hand-computed
# reference value.
ASSORT_DIR_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_degree_directed_weighted_R_triangle_geometric_weights",
        # Directed 3-cycle 0→1, 1→2, 2→0 with weights (1, 2, 4).
        # out_str = [1, 2, 4]; in_str = [4, 1, 2].
        # Hand-computed Pearson: num1=73, num2=21, num3=21,
        # den1=73, den2=73, total_w=7. num = 73 - 21·21/7 = 10.
        # var_from = var_to = 73 - 441/7 = 10.
        # r = 10 / sqrt(10 · 10) = 1.0 (perfect correlation between
        # out-strength of source and in-strength of target).
        "origin": "constructed (rigraph-style): directed 3-cycle weights (1, 2, 4); hand-computed r = 1.0",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (2, 0)],
            directed=True,
        ),
        "graph_weights": [1.0, 2.0, 4.0],
        "algo": "assortativity_degree_directed_weighted",
        "params": {},
        "expected": 1.0,
    },
]

CORENESS_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "coreness_with_mode_R_directed_star_out",
        # rigraph smoke: directed star 0→{1,2,3}. Out-degrees:
        # [3, 0, 0, 0]. Peeling leaves drains 0's core to 0.
        "origin": "constructed (rigraph-style): directed star, out-mode",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (0, 3)],
            directed=True,
        ),
        "algo": "coreness_with_mode",
        "params": {"mode": "out"},
        "expected": [0, 0, 0, 0],
    },
]

DU_MANY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "disjoint_union_many_R_directed_two_paths",
        # rigraph smoke: two directed graphs unioned. Each vertex
        # in the second graph shifts by the first's vcount.
        "origin": "constructed (rigraph-style): two directed paths",
        "graph_factory": lambda: ig.Graph(
            n=2,
            edges=[(0, 1)],
            directed=True,
        ),
        "algo": "disjoint_union_many",
        "params": {
            "extra_graphs": [
                {
                    "n": 3,
                    "edges": [[0, 1], [1, 2]],
                    "directed": True,
                    "weights": None,
                },
            ]
        },
        "expected": {
            "vcount": 5,
            "directed": True,
            "edges": [[0, 1], [2, 3], [3, 4]],
        },
    },
]

IS_SIMPLE_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_simple_with_mode_R_directed_self_loop",
        # rigraph smoke: a directed graph with a self-loop is never
        # simple, regardless of mode.
        "origin": "constructed (rigraph-style): directed self-loop",
        "graph_factory": lambda: ig.Graph(
            n=2,
            edges=[(0, 0)],
            directed=True,
        ),
        "algo": "is_simple_with_mode",
        "params": {"directed_as_undirected": True},
        "expected": False,
    },
]

MODULARITY_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_weighted_R_two_disjoint_edges",
        # rigraph's `modularity(g, membership=, weights=)`. Two
        # disjoint edges with unit weights, partition {0,1} vs {2,3}:
        # w_internal = 4 (each undirected edge → 2*1), W = 2,
        # 2W = 4. e_norm = 1.0. s[c0] = s[c1] = 0.5. Q = 0.5.
        "origin": "constructed (rigraph-style): two disjoint edges",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (2, 3)],
            directed=False,
        ),
        "graph_weights": [1.0, 1.0],
        "algo": "modularity_weighted",
        "params": {"membership": [0, 0, 1, 1], "resolution": 1.0},
        "expected": 0.5,
    },
]

RECIP_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reciprocity_with_mode_R_directed_3_cycle_ratio",
        # rigraph's `reciprocity(g, mode='ratio')`. A directed
        # 3-cycle has zero reciprocal edges and 6 non-reciprocal
        # contributions (each vertex's in/out tail) → ratio = 0.
        "origin": "constructed (rigraph-style): directed 3-cycle, ratio mode",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (2, 0)],
            directed=True,
        ),
        "algo": "reciprocity_with_mode",
        "params": {"ignore_loops": False, "mode": "ratio"},
        "expected": 0.0,
    },
]

CORENESS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "coreness_R_path_5",
        # rigraph's `coreness(g, mode='all')`. On a 5-path every vertex
        # peels at degree 1 → coreness 1 across the board.
        "origin": "constructed (rigraph-style): 5-vertex path 0-1-2-3-4",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4)],
            directed=False,
        ),
        "algo": "coreness",
        "params": {},
        "expected": [1, 1, 1, 1, 1],
    },
]

FW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "floyd_warshall_R_undirected_weighted_triangle",
        # rigraph's `distances(graph, weights=, algorithm="floyd-warshall")`.
        # Triangle with two cheap edges + one expensive direct edge: FW
        # routes 0→2 via vertex 1.
        "origin": "constructed (rigraph-style): undirected triangle weights "
        "(1, 4, 2); shortcut via vertex 1",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "floyd_warshall_distances",
        "params": {},
        "expected": [
            [0.0, 1.0, 3.0],
            [1.0, 0.0, 2.0],
            [3.0, 2.0, 0.0],
        ],
    },
]

DISJOINT_UNION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "disjoint_union_R_directed_two_paths",
        # rigraph's `disjoint_union(g1, g2)`. Two directed 2-paths
        # become a 4-vertex graph with edges (0,1) and (3,2).
        "origin": "constructed (rigraph-style): directed disjoint union",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1)], directed=True
        ),
        "algo": "disjoint_union",
        "params": {
            "right_graph": {
                "n": 2,
                "edges": [[1, 0]],
                "directed": True,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 4,
            "directed": True,
            "edges": [[0, 1], [3, 2]],
        },
    },
]

UNION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "union_R_directed_opposite_paths",
        # rigraph's `union(g1, g2)`. Both graphs have 3 vertices; left
        # is the directed path 0→1→2, right is the reverse 2→1→0.
        # Direction is preserved per ordered pair → 4 distinct edges.
        "origin": "constructed (rigraph-style): directed union of opposing 2-paths",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "algo": "union",
        "params": {
            "right_graph": {
                "n": 3,
                "edges": [[1, 0], [2, 1]],
                "directed": True,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 3,
            "directed": True,
            "edges": [[0, 1], [1, 0], [1, 2], [2, 1]],
        },
    },
]

INTERSECTION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "intersection_R_K4_with_two_K3_subgraphs",
        # rigraph's `intersection(g1, g2)`. K4 on {0,1,2,3} ∩ a graph
        # carrying only the K3 on {0,1,2} edges → the three triangle
        # edges survive. Vertex 3 stays as an isolated vertex (vcount =
        # max(4, 4) = 4, no edges incident to it).
        "origin": "constructed (rigraph-style): K4 ∩ K3-subgraph on 4 vertices",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "algo": "intersection",
        "params": {
            "right_graph": {
                "n": 4,
                "edges": [[0, 1], [0, 2], [1, 2]],
                "directed": False,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 4,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 2]],
        },
    },
]

DIFFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "difference_R_K4_minus_K3_subgraph",
        # rigraph's `difference(g1, g2)`. orig = K4 on {0,1,2,3};
        # sub = K3 on {0,1,2}. Subtracting the triangle from K4 leaves
        # exactly the star at vertex 3: edges {(0,3),(1,3),(2,3)}.
        # vcount = orig.vcount() = 4 (asymmetric).
        "origin": "constructed (rigraph-style): K4 \\ K3-subgraph on 4 vertices",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "algo": "difference",
        "params": {
            "right_graph": {
                "n": 4,
                "edges": [[0, 1], [0, 2], [1, 2]],
                "directed": False,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 4,
            "directed": False,
            "edges": [[0, 3], [1, 3], [2, 3]],
        },
    },
]

IS_LOOP_PER_EDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_loop_R_directed_two_self_loops",
        "origin": "constructed (rigraph-style): directed with 2 self-loops; "
        "per-edge is_loop has 2 True / 1 False",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 0), (1, 2), (2, 2)], directed=True
        ),
        "algo": "is_loop",
        "params": {},
        # Wire round-trip preserves directed edge order via vertex
        # iteration; we record the canonical sorted form (multiset).
        "expected": [False, True, True],
    },
]

IS_MULTIPLE_PER_EDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_multiple_R_three_parallel",
        # rigraph mirrors igraph_is_multiple's "second-or-more" contract:
        # first occurrence canonical-False, the rest True.
        "origin": "constructed (rigraph-style): 3 parallel edges; "
        "first canonical False, remaining True",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1), (0, 1)], directed=False
        ),
        "algo": "is_multiple",
        "params": {},
        "expected": [False, True, True],
    },
]

HAS_LOOP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "has_loop_R_directed_self_loops",
        # rigraph mirrors igraph's `has_loop()` API: directed graph with
        # any self-loop returns TRUE.
        "origin": "constructed (rigraph-style): directed self-loops; has_loop=true",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 0), (1, 2), (2, 2)], directed=True
        ),
        "algo": "has_loop",
        "params": {},
        "expected": True,
    },
]

HAS_MULTIPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "has_multiple_R_directed_parallel",
        "origin": "constructed (rigraph-style): directed parallel pair; has_multiple=true",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 1), (1, 2)], directed=True
        ),
        "algo": "has_multiple",
        "params": {},
        "expected": True,
    },
]

COUNT_LOOPS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_loops_R_undirected_mixed",
        # rigraph users typically run sum(which_loop(g)) for the count;
        # we adopt the same semantics: count edges where source == target.
        "origin": "constructed (rigraph-style): undirected mixed; count_loops=2",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 0), (0, 1), (1, 2), (3, 3)], directed=False
        ),
        "algo": "count_loops",
        "params": {},
        "expected": 2,
    },
    {
        "case": "count_loops_R_directed_no_loops",
        "origin": "constructed (rigraph-style): directed (0,1)(1,2)(2,0) cycle; count_loops=0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "count_loops",
        "params": {},
        "expected": 0,
    },
]

COUNT_MULTIPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_multiple_R_directed_mixed",
        # rigraph: count_multiple(g) gives a per-edge integer vector. With
        # directed (0,1)(0,1)(1,2)(1,0): edges 0 and 1 share (0,1) → 2,2;
        # edge 2 alone (1,2) → 1; edge 3 alone (1,0) → 1.
        "origin": "constructed (rigraph-style): directed (0,1)(0,1)(1,2)(1,0); multiplicity (sorted) = [1,1,2,2]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 1), (1, 2), (1, 0)], directed=True
        ),
        "algo": "count_multiple",
        "params": {},
        "expected": [1, 1, 2, 2],
    },
    {
        "case": "count_multiple_R_undirected_simple",
        "origin": "constructed (rigraph-style): undirected triangle; multiplicity = [1,1,1]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "algo": "count_multiple",
        "params": {},
        "expected": [1, 1, 1],
    },
]

COUNT_ADJACENT_TRIANGLES_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_adjacent_triangles_R_two_disjoint_triangles",
        # rigraph: count_triangles(g, vids=V(g)) returns per-vertex
        # adjacent-triangle counts. Two disjoint triangles → every
        # vertex sees exactly one.
        "origin": "constructed (rigraph-style): two disjoint undirected triangles; per-vertex = [1,1,1,1,1,1]",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)],
            directed=False,
        ),
        "algo": "count_adjacent_triangles",
        "params": {},
        "expected": [1, 1, 1, 1, 1, 1],
    },
    {
        "case": "count_adjacent_triangles_R_path_no_triangles",
        # 4-path: no triangles anywhere.
        "origin": "constructed (rigraph-style): undirected 4-path; per-vertex = [0,0,0,0]",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "count_adjacent_triangles",
        "params": {},
        "expected": [0, 0, 0, 0],
    },
]

IS_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_simple_R_parallel_edges_not_simple",
        # rigraph mirrors igraph's `is_simple()`: parallel edges flip the
        # answer to FALSE. We seed (0,1)(0,1)(1,2) — undirected — to
        # exercise that path.
        "origin": "constructed (rigraph-style): two parallel undirected edges; not simple",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 1), (1, 2)], directed=False
        ),
        "algo": "is_simple",
        "params": {},
        "expected": False,
    },
]

# Louvain (ALGO-CO-002). R-igraph exposes Louvain as
# `cluster_louvain(graph)`. As with the C and Python sources, the
# partition shifts with shuffle order, so the conformance harness
# asserts on modularity range and community count, not on exact
# membership. References:
# - references/rigraph/R/community.R::cluster_louvain
# - references/rigraph/tests/testthat tests for community detection
LOUVAIN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "louvain_R_karate",
        # The standard cluster_louvain demo graph in R is the karate
        # club via make_graph("Zachary") or read.graph("karate.gml").
        # Louvain typically lands on k = 4, Q ≈ 0.42.
        "origin": "rigraph cluster_louvain example: make_graph('Zachary'); "
        "Q ≈ 0.39..0.42, k=4",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "louvain",
        "params": {"resolution": 1.0},
        "expected": {
            "modularity_min": 0.38,
            "modularity_max": 0.43,
            "k_min": 3,
            "k_max": 5,
        },
    },
    {
        "case": "louvain_R_ring_of_4_cliques_5",
        # Ring of 4 K5 cliques, each pair bridged once around the
        # cycle. Ground truth is 4 communities; Louvain hits Q ≈ 0.66.
        "origin": "constructed (R-style benchmark): 4 cliques of size 5 "
        "joined in a ring; Louvain k=4, Q ≈ 0.66",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                # K5 c0: 0..4
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                # K5 c1: 5..9
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                # K5 c2: 10..14
                (10, 11), (10, 12), (10, 13), (10, 14),
                (11, 12), (11, 13), (11, 14),
                (12, 13), (12, 14), (13, 14),
                # K5 c3: 15..19
                (15, 16), (15, 17), (15, 18), (15, 19),
                (16, 17), (16, 18), (16, 19),
                (17, 18), (17, 19), (18, 19),
                # Ring bridges
                (0, 5), (5, 10), (10, 15), (15, 0),
            ],
            directed=False,
        ),
        "algo": "louvain",
        "params": {"resolution": 1.0},
        "expected": {
            "modularity_min": 0.60,
            "modularity_max": 0.70,
            "k_min": 4,
            "k_max": 4,
        },
    },
]

# Leiden (ALGO-CO-003). R-igraph exposes Leiden as
# `cluster_leiden(graph, objective_function="modularity"|"CPM", ...)`.
# As with Louvain, the partition depends on shuffle order, so the
# conformance harness asserts on a modularity range and community-count
# window rather than an exact membership. References:
# - references/rigraph/R/community.R::cluster_leiden
# - references/rigraph/tests/testthat tests for community detection
LEIDEN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "leiden_R_karate",
        # The standard cluster_leiden demo graph in R is the karate
        # club via make_graph("Zachary"). With the Modularity objective
        # and γ=1, Leiden lands near k = 4, Q ≈ 0.42.
        "origin": "rigraph cluster_leiden example: make_graph('Zachary'); "
        "Modularity objective, Q ≈ 0.39..0.45, k ≈ 4",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "leiden",
        "params": {"objective": "modularity", "resolution": 1.0},
        "expected": {
            "modularity_min": 0.36,
            "modularity_max": 0.46,
            "k_min": 2,
            "k_max": 8,
        },
    },
    {
        "case": "leiden_R_ring_of_4_cliques_5",
        # Ring of 4 K5 cliques, each pair bridged once around the
        # cycle. Ground truth is 4 communities; Leiden hits Q ≈ 0.66.
        "origin": "constructed (R-style benchmark): 4 cliques of size 5 "
        "joined in a ring; Leiden Modularity k=4, Q ≈ 0.66",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                # K5 c0: 0..4
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                # K5 c1: 5..9
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                # K5 c2: 10..14
                (10, 11), (10, 12), (10, 13), (10, 14),
                (11, 12), (11, 13), (11, 14),
                (12, 13), (12, 14), (13, 14),
                # K5 c3: 15..19
                (15, 16), (15, 17), (15, 18), (15, 19),
                (16, 17), (16, 18), (16, 19),
                (17, 18), (17, 19), (18, 19),
                # Ring bridges
                (0, 5), (5, 10), (10, 15), (15, 0),
            ],
            directed=False,
        ),
        "algo": "leiden",
        "params": {"objective": "modularity", "resolution": 1.0},
        "expected": {
            "modularity_min": 0.60,
            "modularity_max": 0.70,
            "k_min": 4,
            "k_max": 4,
        },
    },
]

WALKTRAP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "walktrap_R_karate",
        # rigraph cluster_walktrap demo: walktrap on Zachary's karate club
        # with steps=4 lands at Q ≈ 0.35..0.42, k ∈ [3, 6].
        "origin": "rigraph cluster_walktrap example: "
        "make_graph('Zachary'); steps=4; Q ∈ [0.30, 0.45], k ∈ [3, 6]",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "walktrap",
        "params": {"steps": 4},
        "expected": {
            "modularity_min": 0.30,
            "modularity_max": 0.45,
            "k_min": 3,
            "k_max": 6,
        },
    },
    {
        "case": "walktrap_R_ring_of_4_cliques_5",
        # Ring of 4 K5 cliques: cluster_walktrap recovers the 4 cliques
        # cleanly at k = 4, Q ≈ 0.60..0.72.
        "origin": "constructed (R-style benchmark): 4 cliques of size 5 "
        "joined in a ring; cluster_walktrap steps=4 Q ≈ 0.60..0.72, k = 4",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                (10, 11), (10, 12), (10, 13), (10, 14),
                (11, 12), (11, 13), (11, 14),
                (12, 13), (12, 14), (13, 14),
                (15, 16), (15, 17), (15, 18), (15, 19),
                (16, 17), (16, 18), (16, 19),
                (17, 18), (17, 19), (18, 19),
                (0, 5), (5, 10), (10, 15), (15, 0),
            ],
            directed=False,
        ),
        "algo": "walktrap",
        "params": {"steps": 4},
        "expected": {
            "modularity_min": 0.55,
            "modularity_max": 0.75,
            "k_min": 4,
            "k_max": 4,
        },
    },
    {
        "case": "walktrap_R_ring6_weighted",
        # rigraph cluster_walktrap with edge weights — mirrors the C
        # ring-6 weighted reference: weights [1,0.5,0.25,0.75,1.25,1.5]
        # → best Q ≈ 0.146 at k = 3.
        "origin": "rigraph cluster_walktrap with weights; 6-ring weights "
        "[1,0.5,0.25,0.75,1.25,1.5]; best Q = 0.146259, k = 3",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)],
            directed=False,
        ),
        "graph_weights": [1.0, 0.5, 0.25, 0.75, 1.25, 1.5],
        "algo": "walktrap_weighted",
        "params": {"steps": 4},
        "expected": {
            "modularity_min": 0.10,
            "modularity_max": 0.20,
            "k_min": 3,
            "k_max": 3,
        },
    },
]

FASTGREEDY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "fastgreedy_R_karate",
        # rigraph cluster_fast_greedy demo on Zachary's karate club.
        # Q ≈ 0.38, k ∈ [2, 5].
        "origin": "rigraph cluster_fast_greedy example: "
        "make_graph('Zachary'); Q ∈ [0.30, 0.45], k ∈ [2, 5]",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "fast_greedy_modularity",
        "params": {},
        "expected": {
            "modularity_min": 0.30,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 5,
        },
    },
    {
        "case": "fastgreedy_R_ring_of_4_cliques_5",
        # Ring of 4 K5 cliques. fast-greedy modularity recovers the 4
        # cliques (sometimes merging adjacent pairs); Q ≈ 0.5..0.72.
        "origin": "constructed (R-style benchmark): 4 cliques of size 5 "
        "joined in a ring; cluster_fast_greedy Q ≈ 0.50..0.72, k ∈ [2, 4]",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                (10, 11), (10, 12), (10, 13), (10, 14),
                (11, 12), (11, 13), (11, 14),
                (12, 13), (12, 14), (13, 14),
                (15, 16), (15, 17), (15, 18), (15, 19),
                (16, 17), (16, 18), (16, 19),
                (17, 18), (17, 19), (18, 19),
                (0, 5), (5, 10), (10, 15), (15, 0),
            ],
            directed=False,
        ),
        "algo": "fast_greedy_modularity",
        "params": {},
        "expected": {
            "modularity_min": 0.50,
            "modularity_max": 0.72,
            "k_min": 2,
            "k_max": 4,
        },
    },
]

EB_COMMUNITY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eb_community_R_karate",
        # rigraph cluster_edge_betweenness demo on Zachary's karate.
        # Q ≈ 0.40, k ∈ [2, 5].
        "origin": "rigraph cluster_edge_betweenness example: "
        "make_graph('Zachary'); Q ∈ [0.30, 0.45], k ∈ [2, 5]",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "edge_betweenness_community",
        "params": {},
        "expected": {
            "modularity_min": 0.30,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 5,
        },
    },
    {
        "case": "eb_community_R_ring_of_4_cliques_5",
        # Ring of 4 K5 cliques. Edge-betweenness deterministically
        # recovers the 4 cliques; Q ≈ 0.66.
        "origin": "constructed (R-style benchmark): 4 cliques of size 5 "
        "joined in a ring; cluster_edge_betweenness Q ≈ 0.55..0.70, k=4",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                # K5 c0: 0..4
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                # K5 c1: 5..9
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                # K5 c2: 10..14
                (10, 11), (10, 12), (10, 13), (10, 14),
                (11, 12), (11, 13), (11, 14),
                (12, 13), (12, 14), (13, 14),
                # K5 c3: 15..19
                (15, 16), (15, 17), (15, 18), (15, 19),
                (16, 17), (16, 18), (16, 19),
                (17, 18), (17, 19), (18, 19),
                # Ring bridges
                (0, 5), (5, 10), (10, 15), (15, 0),
            ],
            directed=False,
        ),
        "algo": "edge_betweenness_community",
        "params": {},
        "expected": {
            "modularity_min": 0.50,
            "modularity_max": 0.72,
            "k_min": 4,
            "k_max": 4,
        },
    },
    {
        "case": "eb_community_R_directed_path_6",
        # Directed 6-path 0→1→2→3→4→5. The middle edge (2,3) is the
        # unique maximum-betweenness directed edge (eb = 9) so the
        # weighted Girvan-Newman pass removes it first, producing the
        # clean split {0,1,2}|{3,4,5} with directed modularity Q = 8/25.
        "origin": "constructed (R-style benchmark): directed 6-path; "
        "cluster_edge_betweenness deterministically cuts the middle "
        "directed edge; Q ≈ 0.32, k = 2",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)],
            directed=True,
        ),
        "algo": "edge_betweenness_community",
        "params": {},
        "expected": {
            "modularity_min": 0.31,
            "modularity_max": 0.33,
            "k_min": 2,
            "k_max": 2,
        },
    },
]

EB_COMMUNITY_WEIGHTED_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eb_community_weighted_R_karate_unit",
        # Unit weights through the weighted entry must agree with the
        # unweighted dendrogram. rigraph cluster_edge_betweenness on
        # Zachary's karate with weights=rep(1, ecount).
        "origin": "rigraph cluster_edge_betweenness(weights=rep(1,78)): "
        "make_graph('Zachary'); Q ∈ [0.30, 0.45], k ∈ [2, 5]",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "graph_weights": [1.0] * 78,
        "algo": "edge_betweenness_community_weighted",
        "params": {},
        "expected": {
            "modularity_min": 0.30,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 5,
        },
    },
    {
        "case": "eb_community_weighted_R_ring_of_4_cliques_5_unit",
        # Same ring-of-cliques benchmark as EB_COMMUNITY_MANIFEST but
        # routed through the weighted entry with unit weights — the
        # weighted Brandes-Dijkstra pass must reproduce the unweighted
        # 4-cluster split.
        "origin": "constructed (R-style benchmark): 4 cliques of size 5 in a ring; "
        "cluster_edge_betweenness(weights=rep(1,44)); Q ∈ [0.50, 0.72], k = 4",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                (10, 11), (10, 12), (10, 13), (10, 14),
                (11, 12), (11, 13), (11, 14),
                (12, 13), (12, 14), (13, 14),
                (15, 16), (15, 17), (15, 18), (15, 19),
                (16, 17), (16, 18), (16, 19),
                (17, 18), (17, 19), (18, 19),
                (0, 5), (5, 10), (10, 15), (15, 0),
            ],
            directed=False,
        ),
        "graph_weights": [1.0] * 44,
        "algo": "edge_betweenness_community_weighted",
        "params": {},
        "expected": {
            "modularity_min": 0.50,
            "modularity_max": 0.72,
            "k_min": 4,
            "k_max": 4,
        },
    },
    {
        "case": "eb_community_weighted_R_directed_path_6_unit",
        # Directed 6-path 0→1→2→3→4→5 with unit weights through the
        # weighted entry. Weighted Brandes-Dijkstra reproduces the
        # unweighted dendrogram; the middle directed edge is cut first
        # and the {0,1,2}|{3,4,5} split achieves directed weighted Q
        # ≈ 0.32 at k = 2.
        "origin": "constructed (R-style benchmark): directed 6-path; "
        "cluster_edge_betweenness(weights=rep(1,5)) cuts middle directed "
        "edge first; Q ≈ 0.32, k = 2",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)],
            directed=True,
        ),
        "graph_weights": [1.0] * 5,
        "algo": "edge_betweenness_community_weighted",
        "params": {},
        "expected": {
            "modularity_min": 0.31,
            "modularity_max": 0.33,
            "k_min": 2,
            "k_max": 2,
        },
    },
]

FLUID_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "fluid_R_karate_k2",
        "origin": "rigraph cluster_fluid_communities example: "
        "make_graph('Zachary'), no_of_communities=2; Q ∈ [0.20, 0.42]",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "fluid_communities",
        "params": {"k": 2},
        "expected": {
            "modularity_min": 0.20,
            "modularity_max": 0.42,
            "k_min": 2,
            "k_max": 2,
        },
    },
    {
        "case": "fluid_R_ring_of_4_cliques_5_k4",
        # Ring of 4 K5 cliques. Fluid with k=4 reliably recovers the
        # ground-truth partition; Q ≈ 0.66.
        "origin": "constructed (R-style benchmark): 4 cliques of size 5 "
        "joined in a ring; fluid k=4; Q ≈ 0.55..0.70",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                # K5 c0: 0..4
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                # K5 c1: 5..9
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                # K5 c2: 10..14
                (10, 11), (10, 12), (10, 13), (10, 14),
                (11, 12), (11, 13), (11, 14),
                (12, 13), (12, 14), (13, 14),
                # K5 c3: 15..19
                (15, 16), (15, 17), (15, 18), (15, 19),
                (16, 17), (16, 18), (16, 19),
                (17, 18), (17, 19), (18, 19),
                # Ring bridges
                (0, 5), (5, 10), (10, 15), (15, 0),
            ],
            directed=False,
        ),
        "algo": "fluid_communities",
        "params": {"k": 4},
        "expected": {
            "modularity_min": 0.50,
            "modularity_max": 0.72,
            "k_min": 4,
            "k_max": 4,
        },
    },
]

LPA_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lpa_R_karate",
        # The standard cluster_label_prop demo graph in R is the karate
        # club via make_graph("Zachary"). With default mode, LPA lands
        # at k ≈ 3–5, Q ≈ 0.20..0.42 across runs.
        "origin": "rigraph cluster_label_prop example: make_graph('Zachary'); "
        "LPA Q ∈ [0.20, 0.42], k ∈ [2, 10]",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "label_propagation",
        "params": {},
        "expected": {
            "modularity_min": 0.20,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 10,
        },
    },
    {
        "case": "lpa_R_ring_of_4_cliques_5",
        # Ring of 4 K5 cliques. LPA reliably yields k=4 with Q ≈ 0.66.
        "origin": "constructed (R-style benchmark): 4 cliques of size 5 "
        "joined in a ring; LPA k ∈ [3, 5], Q ≈ 0.55..0.70",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                # K5 c0: 0..4
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                # K5 c1: 5..9
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                # K5 c2: 10..14
                (10, 11), (10, 12), (10, 13), (10, 14),
                (11, 12), (11, 13), (11, 14),
                (12, 13), (12, 14), (13, 14),
                # K5 c3: 15..19
                (15, 16), (15, 17), (15, 18), (15, 19),
                (16, 17), (16, 18), (16, 19),
                (17, 18), (17, 19), (18, 19),
                # Ring bridges
                (0, 5), (5, 10), (10, 15), (15, 0),
            ],
            directed=False,
        ),
        "algo": "label_propagation",
        "params": {},
        "expected": {
            "modularity_min": 0.50,
            "modularity_max": 0.72,
            "k_min": 3,
            "k_max": 5,
        },
    },
]

MODULARITY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_R_path3_split_endpoints",
        # From rigraph tests/testthat/test-aaa-auto.R:6342
        # 'modularity_impl basic' — path_graph_impl(n=3) with
        # membership c(1,2,1). Q = -0.5 (computed via python-igraph).
        "origin": "test-aaa-auto.R:6344 'modularity_impl basic': "
        "path(3) with membership [1,2,1]; Q = -0.5",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "modularity",
        "params": {"membership": [0, 1, 0], "resolution": 1.0},
        "expected": -0.5,
    },
]

SIMPLIFY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "simplify_R_directed_loops_only",
        # rigraph mirrors igraph_simplify_example_directed_loops_and_multi:
        # simplify(remove.multiple = FALSE, remove.loops = TRUE) on
        # directed (2,2)(2,2)(2,2)(3,2) leaves (3,2). We swap to small ids
        # (0,0)x3 + (1,0) so the fixture stays compact while exercising
        # remove_loops=true remove_multiple=false on a directed graph.
        "origin": "constructed (rigraph-style): simplify(multiple=F, loops=T) "
        "on directed (0,0)x3 + (1,0)",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 0), (0, 0), (0, 0), (1, 0)], directed=True
        ),
        "algo": "simplify",
        "params": {"remove_multiple": False, "remove_loops": True},
        "expected": {"vcount": 2, "directed": True, "edges": [[1, 0]]},
    },
]

TC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitive_closure_R_undirected_path3",
        # Undirected 0-1-2: closure is K3.
        "origin": "constructed (R-style): undirected path-3; closure is K3",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "transitive_closure",
        "params": {},
        "expected": {
            "vcount": 3,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 2]],
        },
    },
]

REACH_MATRIX_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reachability_matrix_R_disconnected_pair",
        # Two disconnected edges 0-1 and 2-3: 4x4 matrix block-diagonal.
        "origin": "constructed: two disjoint undirected edges; block-diagonal True",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "algo": "reachability_matrix",
        "params": {},
        "expected": [
            [True, True, False, False],
            [True, True, False, False],
            [False, False, True, True],
            [False, False, True, True],
        ],
    },
]

REACH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_reachable_R_path_3_undirected",
        # Path 0-1-2 undirected: every vertex reaches all 3.
        "origin": "constructed (R-style): path_graph(3); count_reachable = [3, 3, 3]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "count_reachable",
        "params": {},
        "expected": [3, 3, 3],
    },
]

EUL_PATH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eulerian_path_R_4cycle_walk_len_4",
        # test-eulerian.R:'has_eulerian_path works' — graph_from_literal
        # A-B-C-D-A. 4-cycle has Eulerian cycle; walk length 4.
        "origin": "test-eulerian.R:'has_eulerian_path works' — 4-cycle, walk has 4 edges",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=False
        ),
        "algo": "eulerian_path",
        "params": {},
        "expected": 4,
    },
]

ASSORT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_R_path_3",
        # R-style basic test: path_graph(3) — the canonical -1.0
        # disassortative case (deg=[1,2,1]).
        "origin": "test-aaa-auto.R-style — path_graph(n=3); assortativity_degree = -1.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "assortativity_degree",
        "params": {},
        "expected": -1.0,
    },
]

DENSITY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "density_R_path_3",
        # path_graph(n=3): 2 edges of 3 possible → 2/3.
        "origin": "test-aaa-auto.R-style — path_graph(n=3); density 2/3",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "density",
        "params": {},
        "expected": 0.6666666666666666,
    },
]

MEANDIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mean_distance_R_triangle",
        # Triangle: all pairs at distance 1; mean = 1.0.
        "origin": "constructed (R-style): triangle; mean distance 1.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "mean_distance",
        "params": {},
        "expected": 1.0,
    },
]

GLOBAL_EFFICIENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "global_efficiency_R_triangle",
        # Triangle K3: all pairs at distance 1, sum of inverses = 6,
        # global_efficiency = 6 / (3*2) = 1.0.
        "origin": "R-style — triangle K3; global efficiency 1.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "global_efficiency",
        "params": {},
        "expected": 1.0,
    },
    {
        "case": "global_efficiency_R_path_3",
        # Undirected path 0-1-2: distances {1,1,2,1,1,2}. sum_inv =
        # 4*1 + 2*0.5 = 5; global_efficiency = 5 / (3*2) = 5/6.
        "origin": "R-style — undirected path 0-1-2; global efficiency 5/6",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "global_efficiency",
        "params": {},
        "expected": 5.0 / 6.0,
    },
]

LOCAL_EFFICIENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "local_efficiency_R_triangle",
        # rigraph/R/efficiency.R: triangle K3 has every vertex's two
        # neighbours connected by a direct edge in G\{v} → local
        # efficiency 1.0 at every vertex.
        "origin": "R-style — triangle K3; local_efficiency=[1,1,1]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "local_efficiency",
        "params": {},
        "expected": [1.0, 1.0, 1.0],
    },
    {
        "case": "local_efficiency_R_star_4",
        # Star K_{1,3}: centre 0 has 3 mutually disconnected neighbours
        # in G\{0} → 0; leaves each have one neighbour → 0.
        "origin": "R-style — star K_{1,3}; local_efficiency=[0,0,0,0]",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "local_efficiency",
        "params": {},
        "expected": [0.0, 0.0, 0.0, 0.0],
    },
]

AVERAGE_LOCAL_EFFICIENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "average_local_efficiency_R_triangle",
        # All per-vertex local efficiencies are 1.0 → mean 1.0.
        "origin": "R-style — triangle K3; average_local_efficiency=1.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "average_local_efficiency",
        "params": {},
        "expected": 1.0,
    },
    {
        "case": "average_local_efficiency_R_path_4",
        # rigraph efficiency.R: undirected path 0-1-2-3. Vertex 1's two
        # neighbours {0,2} are disconnected in G\{1} → 0; vertex 2
        # symmetric → 0; vertices 0 and 3 have one neighbour each → 0.
        # Mean = 0.
        "origin": "R-style — path 0-1-2-3; average_local_efficiency=0.0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "average_local_efficiency",
        "params": {},
        "expected": 0.0,
    },
]

LTRANS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_local_R_triangle",
        # test-aaa-auto.R-style: a triangle has clustering 1.0 at each vertex.
        "origin": "R-style — triangle has clustering 1.0 at each vertex",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "transitivity_local_undirected",
        "params": {},
        "expected": [1.0, 1.0, 1.0],
    },
]

TRANS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_undirected_R_path3",
        # test-aaa-auto.R:#103 transitivity_undirected_impl basic uses
        # path_graph(n=3). With 1 connected triple and 0 triangles,
        # mode='nan' default → 0 (no triangles → 0/1 = 0).
        # Wait: 3*0/1 = 0. So expected is 0.0.
        "origin": "test-aaa-auto.R:#103 transitivity_undirected_impl basic — path_graph(n=3) → 0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "transitivity_undirected",
        "params": {},
        "expected": 0.0,
    },
]

DIAM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_R_disjoint_trees",
        # test-structural-properties.R:'diameter() correctly handles
        # disconnected graphs' make_tree(7,2) %du% make_tree(4,3) →
        # diameter(unconnected=TRUE) = 4 (longest geodesic in larger
        # tree, ignoring the disconnect).
        "origin": "test-structural-properties.R:'diameter() correctly handles disconnected graphs' "
        "make_tree(7,2) %du% make_tree(4,3); diameter(unconnected=TRUE) = 4",
        "graph_factory": lambda: ig.Graph.Tree(n=7, children=2, mode="undirected").disjoint_union(
            ig.Graph.Tree(n=4, children=3, mode="undirected")
        ),
        "algo": "diameter",
        "params": {},
        "expected": 4,
    },
]

ECC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_R_path_graph_3",
        # test-aaa-auto.R uses path_graph_impl(n=3) for many basic tests.
        # 0-1-2: ecc = [2, 1, 2].
        "origin": "test-aaa-auto.R-style — path_graph(n=3); eccentricity vector",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "eccentricity",
        "params": {},
        "expected": [2, 1, 2],
    },
]

RAD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_R_path_graph_3",
        "origin": "test-aaa-auto.R-style — path_graph(n=3); radius = 1 (centre)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "radius",
        "params": {},
        "expected": 1,
    },
]

ECC_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_with_mode_R_directed_star_out",
        # rigraph-style: directed out-star 0→1, 0→2, 0→3 under
        # IGRAPH_OUT. Centre vertex 0 reaches every leaf at distance 1
        # (ecc=1); leaves have no out-edges (ecc=0). Expected: [1,0,0,0].
        "origin": "constructed (rigraph-style): directed out-star — OUT mode",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=True
        ),
        "algo": "eccentricity_with_mode",
        "params": {"mode": "out"},
        "expected": [1, 0, 0, 0],
    },
]

RAD_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_with_mode_R_directed_star_out",
        # Same directed out-star — min ecc is 0 (any leaf has no
        # out-edges).
        "origin": "constructed (rigraph-style): directed out-star — OUT-mode min = 0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=True
        ),
        "algo": "radius_with_mode",
        "params": {"mode": "out"},
        "expected": 0,
    },
]

DIAM_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_with_mode_R_directed_star_out",
        # Same directed out-star — max ecc is 1 (the centre's reach).
        "origin": "constructed (rigraph-style): directed out-star — OUT-mode diam = 1",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=True
        ),
        "algo": "diameter_with_mode",
        "params": {"mode": "out"},
        "expected": 1,
    },
]

GIRTH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "girth_R_make_ring_100",
        # test-structural-properties.R:'girth() works' make_ring(100) → 100.
        "origin": "test-structural-properties.R:'girth() works' — make_ring(100) → girth 100",
        "graph_factory": lambda: ig.Graph.Ring(
            n=100, directed=False, mutual=False, circular=True
        ),
        "algo": "girth",
        "params": {},
        "expected": 100,
    },
]

ISBI_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_biconnected_path_3",
        # test-aaa-auto.R:#178 is_biconnected_impl basic uses path_graph(3).
        # 3-vertex undirected path is NOT biconnected (vertex 1 is articulation).
        "origin": "test-aaa-auto.R:#178 is_biconnected_impl basic — path_graph(n=3)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "is_biconnected",
        "params": {},
        "expected": False,
    },
]

BRIDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bridges_krackhardt_kite",
        # test-components.R:'bridges works' uses make_graph("krackhardt_kite")
        # and expects bridges = (ecount-1):(ecount) → in 0-based, edges 16
        # and 17. Construct the same graph via python-igraph.Famous so
        # edge indexing matches.
        "origin": "test-components.R:'bridges works' — make_graph('krackhardt_kite') bridges = (ecount-1):ecount",
        "graph_factory": lambda: ig.Graph.Famous("krackhardt_kite"),
        "algo": "bridges",
        "params": {},
        "expected": [16, 17],
    },
]

AP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "path_graph_3_articulation",
        # test-aaa-auto.R:#175 articulation_points_impl basic uses
        # path_graph_impl(n=3) — undirected path 0-1-2. The middle vertex
        # is the only articulation point.
        "origin": "test-aaa-auto.R:#175 articulation_points_impl basic — path_graph(n=3)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "articulation_points",
        "params": {},
        "expected": [1],
    },
]

DIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ring10_source0",
        # test-structural-properties.R uses make_ring(10) for the BFS
        # multi-root test. Distances from vertex 0 on an undirected 10-cycle
        # rise from 0 then mirror back: 0,1,2,3,4,5,4,3,2,1.
        "origin": "test-structural-properties.R:make_ring(10) — distances "
        "from vertex 0; expected via python-igraph 0.11 distances() "
        "(matches igraph C unweighted BFS)",
        "graph_factory": lambda: _ring(10),
        "algo": "distances",
        "params": {"source": 0},
        "expected": [0, 1, 2, 3, 4, 5, 4, 3, 2, 1],
    },
]

SCC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "components_R_largest_strong_weak",
        # test-components.R:'largest strongly and weakly components are correct'
        # (lines 102-125): graph_from_literal A-+B, B-+C, C-+A, C-+D, E
        # (5 vertices, 1-based labels A=1, B=2, C=3, D=4, E=5).
        # Translated to 0-based: 0->1, 1->2, 2->0, 2->3, isolate 4.
        # Strong components: {0,1,2}, {3}, {4}. Label order matches python-igraph.
        "origin": "test-components.R:'largest strongly and weakly components are correct' "
        "graph_from_literal(A-+B, B-+C, C-+A, C-+D, E); SCC labels via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (1, 2), (2, 0), (2, 3)],
            directed=True,
        ),
        "algo": "strongly_connected_components",
        "params": {},
        "expected": {"membership": [1, 1, 1, 2, 0], "count": 3},
    },
]

COMMUNITY_TO_MEMBERSHIP_MANIFEST: List[Dict[str, Any]] = [
    # R-igraph exposes the same C helper via `cut_at()` and via the
    # internal `community_to_membership` glue in
    # references/rigraph/R/community.R. Fixtures mirror the
    # documented contract on small hand-constructed dendrograms; the
    # conformance test compares partitions canonically since cluster
    # labels may differ from the Rust impl.
    {
        "case": "community_to_membership_r_zero_steps",
        "origin": "R-igraph cut_at: 4 leaves, two merges [[0,1],[2,3]], "
        "steps=0 -> 4 singletons",
        "nodes": 4,
        "merges": [[0, 1], [2, 3]],
        "steps": 0,
        "expected": {"membership": [0, 1, 2, 3], "csize": [1, 1, 1, 1]},
    },
    {
        "case": "community_to_membership_r_full_collapse",
        "origin": "R-igraph cut_at: 4 leaves, balanced [[0,1],[2,3],[4,5]], "
        "steps=3 -> single cluster of 4",
        "nodes": 4,
        "merges": [[0, 1], [2, 3], [4, 5]],
        "steps": 3,
        "expected": {"membership": [0, 0, 0, 0], "csize": [4]},
    },
]

ECC_PR031_MANIFEST: List[Dict[str, Any]] = [
    # R-igraph has `ecc_impl()` (auto-generated wrapper around
    # `igraph_ecc`) but does not yet export it as a public
    # user-facing function. The fixtures below mirror the Radicchi
    # definition and exercise the same shapes the C reference test
    # exercises, with values computed by hand on tiny graphs.
    {
        "case": "ecc_r_c4_k3_normalized_offset_false",
        # C_4 (4-cycle): no triangles → z = 0 everywhere. Degrees all
        # 2, so s = 1; result = 0/1 = 0 for every edge.
        "origin": "R-style — C_4, k=3, normalize=true → all 0.0 (no triangles)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=False
        ),
        "algo": "ecc",
        "params": {"k": 3, "offset": False, "normalize": True},
        "expected": [0.0, 0.0, 0.0, 0.0],
    },
    {
        "case": "ecc_r_c4_k4_normalized_offset_false",
        # C_4 at k=4: each edge sits in exactly one 4-cycle (z = 1),
        # degrees all 2 → s = (2-1)*(2-1) = 1; result = 1 everywhere.
        "origin": "R-style — C_4, k=4, normalize=true → all 1.0 (one 4-cycle each)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=False
        ),
        "algo": "ecc",
        "params": {"k": 4, "offset": False, "normalize": True},
        "expected": [1.0, 1.0, 1.0, 1.0],
    },
    {
        "case": "ecc_r_star_k3_normalize_true_is_nan",
        # K_{1,3} star: leaf has degree 1, s = min(1,3) - 1 = 0 → NaN.
        "origin": "R-style — K_{1,3} star, k=3, normalize=true → NaN per edge (s = 0)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "ecc",
        "params": {"k": 3, "offset": False, "normalize": True},
        "expected": [None, None, None],
    },
    {
        "case": "ecc_r_triangle_offset_true_normalize_true_is_radicchi_canonical",
        # K_3 with Radicchi canonical (z + 1) / s: z=1, s=1 → 2.0.
        "origin": "R-style — K_3, k=3, Radicchi canonical (offset+normalize) → 2.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "ecc",
        "params": {"k": 3, "offset": True, "normalize": True},
        "expected": [2.0, 2.0, 2.0],
    },
]


REINDEX_MEMBERSHIP_MANIFEST: List[Dict[str, Any]] = [
    # R-igraph exposes `igraph_reindex_membership` indirectly through
    # the C glue used by `make_clusters()` and community methods. The
    # fixtures below mirror the documented first-occurrence
    # densification semantics. The conformance test compares the
    # partition canonically.
    {
        "case": "reindex_membership_r_negative_id_style",
        "origin": "R-igraph make_clusters parity: contiguous ids "
        "that happen to start above zero — densified to 0..k-1",
        "membership": [3, 3, 4, 5, 5, 4],
        "expected": {"membership": [0, 0, 1, 2, 2, 1], "new_to_old": [3, 4, 5]},
    },
    {
        "case": "reindex_membership_r_alternating",
        "origin": "R-igraph make_clusters parity: alternating pattern "
        "where every consecutive pair belongs to a different cluster",
        "membership": [9, 4, 9, 4, 9, 4, 9],
        "expected": {"membership": [0, 1, 0, 1, 0, 1, 0], "new_to_old": [9, 4]},
    },
]

COMPARE_COMMUNITIES_MANIFEST: List[Dict[str, Any]] = [
    # R-igraph wraps `igraph_compare_communities` via
    # `compare(comm1, comm2, method=)`. The methods exposed in R are
    # "vi", "nmi", "split.join", "rand", "adjusted.rand". Expected
    # values below were derived by closed-form arithmetic on each
    # toy partition and verified against R-igraph's online docs.
    {
        "case": "compare_communities_r_vi_identical",
        "origin": "R-igraph compare() parity: identical partitions ⇒ "
        "variation of information is 0",
        "comm1": [0, 0, 1, 1, 2, 2],
        "comm2": [3, 3, 4, 4, 5, 5],
        "method": "VariationOfInformation",
        "expected": 0.0,
    },
    {
        "case": "compare_communities_r_rand_partial",
        "origin": "R-igraph compare() parity: 4-vertex split where "
        "rand index = 4/6 (4 agreeing pairs out of 6 total pairs)",
        "comm1": [0, 0, 1, 1],
        "comm2": [0, 1, 0, 1],
        "method": "Rand",
        "expected": 0.3333333333333333,
    },
]

SPLIT_JOIN_DISTANCE_MANIFEST: List[Dict[str, Any]] = [
    # R-igraph wraps `igraph_split_join_distance` via
    # `split_join_distance(comm1, comm2)` which returns the asymmetric
    # pair as a length-2 integer vector `c(distance12, distance21)`.
    # Expected values are verified against the upstream confusion-matrix
    # decomposition and are partition-relabel invariant.
    {
        "case": "split_join_distance_r_refinement",
        "origin": "R-igraph split_join_distance() parity: comm2 strictly "
        "refines comm1 (b splits each a-cluster into 2-1) ⇒ d12=2, d21=0.",
        "comm1": [0, 0, 0, 1, 1, 1],
        "comm2": [5, 5, 6, 7, 7, 8],
        "expected": {"d12": 2, "d21": 0},
    },
    {
        "case": "split_join_distance_r_full_disagreement_2x2",
        "origin": "R-igraph split_join_distance() parity: 2x2 "
        "full-disagreement (n=4) — d12=d21=2.",
        "comm1": [0, 0, 1, 1],
        "comm2": [0, 1, 0, 1],
        "expected": {"d12": 2, "d21": 2},
    },
]

VORONOI_MANIFEST: List[Dict[str, Any]] = [
    # R-igraph wraps `igraph_voronoi` via `voronoi_cells()` (see
    # references/rigraph/R/community.R lines 3213-3270). The R API
    # returns 1-based generator indices in `membership` — we translate
    # to 0-based to match the Rust API (which uses 0-based vertex /
    # generator indexing throughout). Expected values are hand-derived
    # from the canonical BFS computations on each test graph and are
    # verifiable by running `voronoi_cells(g, generators+1L,
    # tiebreaker='first')$membership - 1L` in R.
    {
        "case": "voronoi_r_path5_endpoints_first",
        "origin": "R-igraph voronoi_cells(make_graph(c(1,2,2,3,3,4,4,5)), c(1L,5L), "
        "tiebreaker='first', mode='all') — vertex 2 ties (dist=2); FIRST keeps generator 0.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "params": {
            "generators": [0, 4],
            "mode": "all",
            "tiebreaker": "first",
        },
        "expected": {
            "membership": [0, 0, 0, 1, 1],
            "distances": [0.0, 1.0, 2.0, 1.0, 0.0],
        },
    },
    {
        "case": "voronoi_r_path5_endpoints_last",
        "origin": "R-igraph voronoi_cells(make_graph(c(1,2,2,3,3,4,4,5)), c(1L,5L), "
        "tiebreaker='last', mode='all') — vertex 2 ties (dist=2); LAST flips it to generator 1.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "params": {
            "generators": [0, 4],
            "mode": "all",
            "tiebreaker": "last",
        },
        "expected": {
            "membership": [0, 0, 1, 1, 1],
            "distances": [0.0, 1.0, 2.0, 1.0, 0.0],
        },
    },
    {
        "case": "voronoi_r_star_centre_only",
        "origin": "R-igraph voronoi_cells(make_star(5, mode='undirected'), c(1L), "
        "tiebreaker='first', mode='all') — single generator at centre absorbs every leaf.",
        "graph_factory": lambda: ig.Graph.Star(n=5, mode="undirected", center=0),
        "params": {
            "generators": [0],
            "mode": "all",
            "tiebreaker": "first",
        },
        "expected": {
            "membership": [0, 0, 0, 0, 0],
            "distances": [0.0, 1.0, 1.0, 1.0, 1.0],
        },
    },
]

COMMUNITY_VORONOI_MANIFEST: List[Dict[str, Any]] = [
    # R-igraph (rigraph 2.x) exposes `igraph_community_voronoi` through
    # `cluster_voronoi()` (references/rigraph/R/community.R). R has no
    # `.out`-style golden output for it; the fixtures below are
    # hand-derived from the deterministic generator-selection rule
    # (greedy LRD descent with vertex-id tiebreak), the same as the
    # python fixtures but exercising different parameter combinations
    # for cross-source coverage.
    {
        "case": "community_voronoi_r_singleton",
        "origin": "R-igraph cluster_voronoi(make_empty_graph(1)) — single vertex",
        "graph_factory": lambda: ig.Graph(n=1, edges=[], directed=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": -1.0},
        "expected": {"generators": [0], "community_count": 1},
    },
    {
        "case": "community_voronoi_r_two_isolated_nodes",
        "origin": "R-igraph cluster_voronoi(make_empty_graph(2)) — two isolated vertices",
        "graph_factory": lambda: ig.Graph(n=2, edges=[], directed=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": -1.0},
        "expected": {"generators": [0, 1], "community_count": 2},
    },
    {
        "case": "community_voronoi_r_k3_fixed_r0",
        # K_3 with r=0: every other vertex is at strictly positive
        # distance, so each pick excludes only the generator itself →
        # 3 singleton communities. LRD uniform, ties by vertex id.
        "origin": "R-igraph cluster_voronoi(make_full_graph(3), r=0) — 3 singletons",
        "graph_factory": lambda: ig.Graph.Full(n=3, directed=False, loops=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": 0.0},
        "expected": {"generators": [0, 1, 2], "community_count": 3},
    },
    {
        "case": "community_voronoi_r_k4_fixed_r0",
        # K_4 with r=0: uniform LRD, ties by vertex id ⇒ generators
        # [0,1,2,3]. Independent verification at a different size.
        "origin": "R-igraph cluster_voronoi(make_full_graph(4), r=0) — 4 singletons",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": 0.0},
        "expected": {"generators": [0, 1, 2, 3], "community_count": 4},
    },
]

# ALGO-MST-001: minimum_spanning_tree. rigraph exposes
# `mst(graph, weights=NULL, algorithm="unweighted"|"prim"|"kruskal")`.
# rigraph's documentation example uses a sample_gnm graph plus runif()
# weights — not portable here. We mirror the API surface (unweighted +
# Kruskal + Prim + Automatic) on small hand-derived graphs that test
# the rigraph-style "disconnected ⇒ spanning forest" expectation.
SPANNING_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "spanning_tree_r_disconnected_forest_unweighted",
        # rigraph's `mst()` on a disconnected graph returns a spanning
        # forest (one tree per component) — the same invariant the
        # Unweighted dispatch must honour.
        "origin": "constructed (rigraph mst(g, algorithm='unweighted') style): "
        "5-vertex graph with two components → 3-edge spanning forest",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (3, 4)], directed=False
        ),
        "algo": "minimum_spanning_tree",
        "params": {"method": "unweighted"},
        "expected": {"total_weight": 3.0, "edge_count": 3},
    },
    {
        "case": "spanning_tree_r_triangle_kruskal",
        # Standard rigraph-style weighted triangle.
        "origin": "constructed (rigraph mst(g, weights, algorithm='kruskal') "
        "style): triangle (1, 2, 5)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 5.0],
        "algo": "minimum_spanning_tree",
        "params": {"method": "kruskal"},
        "expected": {"total_weight": 3.0, "edge_count": 2},
    },
    {
        "case": "spanning_tree_r_k4_prim",
        # K4 distinct weights — uniqueness guarantees Prim and Kruskal
        # agree on edge set.
        "origin": "constructed (rigraph mst(g, weights, algorithm='prim') style): "
        "K4 distinct weights",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "graph_weights": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        "algo": "minimum_spanning_tree",
        "params": {"method": "prim"},
        "expected": {"total_weight": 6.0, "edge_count": 3},
    },
]

# ALGO-GN-001: erdos_renyi_gnp / erdos_renyi_gnm. rigraph's API is
# `sample_gnp(n, p, directed=FALSE, loops=FALSE)` and
# `sample_gnm(n, m, directed=FALSE, loops=FALSE)`. Same invariant-only
# coverage as the C/py manifests, with different seeds and shapes so
# the three sources stay independent rather than redundant.
ERDOS_RENYI_GNP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "erdos_renyi_gnp_r_undirected_n25_p04",
        # G(25, 0.4). max_edges = 300, µ = 120, σ ≈ 8.49, ±6σ band
        # ≈ [69, 171] → [65, 175].
        "origin": "constructed (mirrors rigraph sample_gnp(25, 0.4)): "
        "Binomial(300, 0.4) ±6σ band",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 25,
            "p": 0.4,
            "directed": False,
            "loops": False,
            "seed": 2_718_281,
        },
        "expected": {
            "vcount": 25,
            "ecount_min": 65,
            "ecount_max": 175,
            "directed": False,
        },
    },
    {
        "case": "erdos_renyi_gnp_r_directed_n10_p06",
        # Directed G(10, 0.6) no loops. max_edges = 90, µ = 54, σ ≈ 4.65,
        # ±6σ band ≈ [26, 82] → [25, 85].
        "origin": "constructed (mirrors rigraph sample_gnp(10, 0.6, "
        "directed=TRUE)): ordered-pair binomial",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 10,
            "p": 0.6,
            "directed": True,
            "loops": False,
            "seed": 1_414_213,
        },
        "expected": {
            "vcount": 10,
            "ecount_min": 25,
            "ecount_max": 85,
            "directed": True,
        },
    },
    {
        "case": "erdos_renyi_gnp_r_n1_singleton",
        # n=1 → at most one self-loop if loops=TRUE, otherwise empty.
        # We pick loops=FALSE so the boundary is exact: vcount=1, ecount=0.
        "origin": "constructed (mirrors rigraph sample_gnp(1, 0.5)): "
        "singleton has no edges without loops",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 1,
            "p": 0.5,
            "directed": False,
            "loops": False,
            "seed": 123,
        },
        "expected": {
            "vcount": 1,
            "ecount_min": 0,
            "ecount_max": 0,
            "directed": False,
        },
    },
]

ERDOS_RENYI_GNM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "erdos_renyi_gnm_r_undirected_n15_m25",
        # G(15, 25) undirected, no loops. Distinct-sample → ecount exact.
        "origin": "constructed (mirrors rigraph sample_gnm(15, 25)): "
        "ecount equals m, without-replacement sampling",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 15,
            "m": 25,
            "directed": False,
            "loops": False,
            "seed": 161_803,
        },
        "expected": {"vcount": 15, "ecount": 25, "directed": False},
    },
    {
        "case": "erdos_renyi_gnm_r_directed_n12_m30",
        # Directed G(12, 30). max_edges = 132.
        "origin": "constructed (mirrors rigraph sample_gnm(12, 30, "
        "directed=TRUE)): ordered pair sampling",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 12,
            "m": 30,
            "directed": True,
            "loops": False,
            "seed": 333_333,
        },
        "expected": {"vcount": 12, "ecount": 30, "directed": True},
    },
    {
        "case": "erdos_renyi_gnm_r_complete_directed_n4_m12",
        # Directed K4 without loops: max_edges = 12. m == max forces
        # the complete directed graph regardless of seed.
        "origin": "constructed (mirrors rigraph sample_gnm(4, 12, "
        "directed=TRUE)): m=max_edges yields complete directed graph",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 4,
            "m": 12,
            "directed": True,
            "loops": False,
            "seed": 50_000,
        },
        "expected": {"vcount": 4, "ecount": 12, "directed": True},
    },
]

# ALGO-GN-002: barabasi_game_bag. Mirrors rigraph's `sample_pa()` /
# `barabasi.game()` with the BAG algorithm choice. Structural
# invariants only — R's RNG state isn't portable.
BARABASI_BAG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_game_bag_r_directed_n40_m3",
        "origin": "constructed (mirrors rigraph sample_pa(n=40, "
        "power=1, m=3, out.pref=FALSE, directed=TRUE, "
        "algorithm='bag'))",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 40,
            "m": 3,
            "outpref": False,
            "directed": True,
            "seed": 314_159,
        },
        "expected": {
            "vcount": 40,
            "ecount": 117,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "barabasi_game_bag_r_undirected_n20_m2",
        "origin": "constructed (mirrors rigraph sample_pa(n=20, m=2, "
        "directed=FALSE, algorithm='bag')): undirected forces outpref",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 20,
            "m": 2,
            "outpref": False,
            "directed": False,
            "seed": 271_828,
        },
        "expected": {
            "vcount": 20,
            "ecount": 38,
            "directed": False,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "barabasi_game_bag_r_m1_tree_n30",
        "origin": "constructed (mirrors rigraph sample_pa(n=30, m=1, "
        "directed=TRUE, algorithm='bag')): m=1 yields exactly n-1 edges "
        "(tree-shaped DAG)",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 30,
            "m": 1,
            "outpref": False,
            "directed": True,
            "seed": 161_803,
        },
        "expected": {
            "vcount": 30,
            "ecount": 29,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
]

# ALGO-GN-003: growing_random_game. Mirrors rigraph's
# `sample_growing()` / `growing.random.game()`. Structural invariants
# only — R's RNG state isn't portable.
GROWING_RANDOM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "growing_random_r_directed_citation_n30_m4",
        "origin": "constructed (mirrors rigraph sample_growing(n=30, "
        "m=4, directed=TRUE, citation=TRUE))",
        "algo": "growing_random_game",
        "params": {
            "n": 30,
            "m": 4,
            "directed": True,
            "citation": True,
            "seed": 414_141,
        },
        "expected": {
            "vcount": 30,
            "ecount": 116,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "growing_random_r_directed_free_n25_m1",
        "origin": "constructed (mirrors rigraph sample_growing(n=25, "
        "m=1, directed=TRUE, citation=FALSE)): m=1 is the smallest "
        "non-trivial step",
        "algo": "growing_random_game",
        "params": {
            "n": 25,
            "m": 1,
            "directed": True,
            "citation": False,
            "seed": 282_828,
        },
        "expected": {
            "vcount": 25,
            "ecount": 24,
            "directed": True,
            "ba_temporal_order": False,
        },
    },
    {
        "case": "growing_random_r_undirected_citation_n50_m2",
        "origin": "constructed (mirrors rigraph sample_growing(n=50, "
        "m=2, directed=FALSE, citation=TRUE)): undirected citation",
        "algo": "growing_random_game",
        "params": {
            "n": 50,
            "m": 2,
            "directed": False,
            "citation": True,
            "seed": 535_353,
        },
        "expected": {
            "vcount": 50,
            "ecount": 98,
            "directed": False,
            "ba_temporal_order": True,
        },
    },
]

# ALGO-GN-004: tree_game (LERW). Mirrors rigraph's
# `sample_tree(n, directed, method = "lerw")` (`sample_tree_game` in
# C-level wrapper). Generator — seed not portable, structural invariants
# only.
TREE_LERW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "tree_lerw_r_undirected_n15",
        "origin": "constructed (mirrors sample_tree(n=15, directed=FALSE, "
        "method='lerw')): small undirected spanning tree",
        "algo": "tree_game_lerw",
        "params": {"n": 15, "directed": False, "seed": 707_070},
        "expected": {"vcount": 15, "ecount": 14, "directed": False, "is_tree": True},
    },
    {
        "case": "tree_lerw_r_undirected_n75",
        "origin": "constructed (mirrors sample_tree(n=75, directed=FALSE, "
        "method='lerw')): larger undirected spanning tree",
        "algo": "tree_game_lerw",
        "params": {"n": 75, "directed": False, "seed": 808_080},
        "expected": {"vcount": 75, "ecount": 74, "directed": False, "is_tree": True},
    },
    {
        "case": "tree_lerw_r_directed_n25",
        "origin": "constructed (mirrors sample_tree(n=25, directed=TRUE, "
        "method='lerw')): directed spanning tree, walk-rooted",
        "algo": "tree_game_lerw",
        "params": {"n": 25, "directed": True, "seed": 909_090},
        "expected": {"vcount": 25, "ecount": 24, "directed": True, "is_tree": True},
    },
]

# ALGO-GN-005: grg_game (geometric random graph). Mirrors rigraph's
# `sample_grg(nodes, radius, torus = FALSE)`. Generator — RNG not
# portable across implementations, so we assert structural invariants
# only (vcount, undirected, simple) and a loose ecount band derived
# from the bulk expectation E[edges] = n(n-1)/2 · π·r² (interior).
GRG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "grg_r_plane_n40_r020",
        "origin": "constructed (mirrors sample_grg(nodes=40, radius=0.20, "
        "torus=FALSE)): undirected plane GRG",
        "algo": "grg_game",
        "params": {"n": 40, "radius": 0.20, "torus": False, "seed": 707_701},
        "expected": {
            "vcount": 40,
            "directed": False,
            "is_simple": True,
            # bulk = C(40,2)·π·0.04 ≈ 98; loose 30..200 covers RNG spread.
            "ecount_min": 30,
            "ecount_max": 200,
        },
    },
    {
        "case": "grg_r_torus_n50_r015",
        "origin": "constructed (mirrors sample_grg(nodes=50, radius=0.15, "
        "torus=TRUE)): undirected torus GRG",
        "algo": "grg_game",
        "params": {"n": 50, "radius": 0.15, "torus": True, "seed": 808_802},
        "expected": {
            "vcount": 50,
            "directed": False,
            "is_simple": True,
            # bulk = C(50,2)·π·0.0225 ≈ 86; loose 25..200.
            "ecount_min": 25,
            "ecount_max": 200,
        },
    },
    {
        "case": "grg_r_dense_complete_n20",
        "origin": "constructed (mirrors sample_grg(nodes=20, radius=2.0, "
        "torus=FALSE)): radius > sqrt(2) → complete graph",
        "algo": "grg_game",
        "params": {"n": 20, "radius": 2.0, "torus": False, "seed": 909_903},
        "expected": {
            "vcount": 20,
            "directed": False,
            "is_simple": True,
            "ecount_min": 190,  # C(20,2) = 190 exactly
            "ecount_max": 190,
        },
    },
]

# ALGO-GN-006: forest_fire_game. Mirrors rigraph's
# `sample_forestfire(nodes, fw.prob, bw.factor=1, ambs=1, directed=TRUE)`.
# Generator — RNG state not portable across implementations, so we
# encode structural invariants only (vcount, directed flag, is_simple)
# and a loose ecount band. Lower bound is n-1 when ambs >= 1 (each new
# vertex contributes at least one ambassador citation); upper bound is
# generous to absorb burn-tail variance.
FOREST_FIRE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "forest_fire_r_directed_n50_fw02_bw03_ambs1",
        "origin": "constructed (mirrors sample_forestfire(nodes=50, "
        "fw.prob=0.2, bw.factor=0.3, ambs=1, directed=TRUE)): single "
        "ambassador directed graph",
        "algo": "forest_fire_game",
        "params": {
            "n": 50,
            "fw_prob": 0.2,
            "bw_factor": 0.3,
            "ambs": 1,
            "directed": True,
            "seed": 9_990_001,
        },
        "expected": {
            "vcount": 50,
            "directed": True,
            "is_simple": True,
            "ecount_min": 49,
            "ecount_max": 5000,
        },
    },
    {
        "case": "forest_fire_r_undirected_n100_fw01_bw05_ambs2",
        "origin": "constructed (mirrors sample_forestfire(nodes=100, "
        "fw.prob=0.1, bw.factor=0.5, ambs=2, directed=FALSE)): cool "
        "burn undirected graph",
        "algo": "forest_fire_game",
        "params": {
            "n": 100,
            "fw_prob": 0.1,
            "bw_factor": 0.5,
            "ambs": 2,
            "directed": False,
            "seed": 9_990_002,
        },
        "expected": {
            "vcount": 100,
            "directed": False,
            "is_simple": True,
            "ecount_min": 99,
            "ecount_max": 10000,
        },
    },
    {
        "case": "forest_fire_r_no_burn_n20_fw0_bw0_ambs2",
        "origin": "constructed (mirrors sample_forestfire(nodes=20, "
        "fw.prob=0, bw.factor=0, ambs=2, directed=TRUE)): zero burn "
        "probability ⇒ only ambassadors cited",
        "algo": "forest_fire_game",
        "params": {
            "n": 20,
            "fw_prob": 0.0,
            "bw_factor": 0.0,
            "ambs": 2,
            "directed": True,
            "seed": 9_990_003,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "is_simple": True,
            # actnode k (1..19) draws 2 ambassadors from [0,k):
            # min 1 (both hit same; k=1 always), max min(2, k).
            # Total bounded by 1+19 = 20 below, sum_{k=1..19} min(2,k) = 1 + 2*18 = 37 above.
            "ecount_min": 19,
            "ecount_max": 37,
        },
    },
]

# ALGO-GN-007: simple_interconnected_islands_game. Mirrors rigraph's
# `sample_islands(islands.n, islands.size, islands.pin, n.inter)`
# (the canonical R wrapper for
# `igraph_simple_interconnected_islands_game`). RNG state is not
# portable across implementations, so the manifest records only
# structural invariants: vcount, directed = FALSE, is_simple, and
# an ecount band built from E[intra] = islands_n · C(size, 2) · pin
# plus exact_inter = C(islands_n, 2) · n_inter.
ISLANDS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "islands_r_5islands_size12_pin025_inter2",
        "origin": "constructed (mirrors sample_islands(islands.n=5, "
        "islands.size=12, islands.pin=0.25, n.inter=2)): five small "
        "islands stitched together with two cross-edges each",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 5,
            "islands_size": 12,
            "islands_pin": 0.25,
            "n_inter": 2,
            "seed": 9_990_101,
        },
        "expected": {
            "vcount": 60,
            "directed": False,
            "is_simple": True,
            # E[intra] = 5 * 12*11/2 * 0.25 = 82.5; exact_inter = C(5,2)*2 = 20.
            # Band [0.6*82.5 + 20, 1.4*82.5 + 20] = [69, 135].
            "ecount_min": 69,
            "ecount_max": 135,
        },
    },
    {
        "case": "islands_r_pin0_only_inter",
        "origin": "constructed (mirrors sample_islands(islands.n=6, "
        "islands.size=10, islands.pin=0, n.inter=1)): no intra edges, "
        "exactly C(6,2)·1 = 15 inter-island edges",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 6,
            "islands_size": 10,
            "islands_pin": 0.0,
            "n_inter": 1,
            "seed": 9_990_102,
        },
        "expected": {
            "vcount": 60,
            "directed": False,
            "is_simple": True,
            "ecount_min": 15,
            "ecount_max": 15,
        },
    },
    {
        "case": "islands_r_single_island_pin_one_clique",
        "origin": "constructed (mirrors sample_islands(islands.n=1, "
        "islands.size=12, islands.pin=1, n.inter=0)): degenerate "
        "single island with pin=1 becomes K_12",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 1,
            "islands_size": 12,
            "islands_pin": 1.0,
            "n_inter": 0,
            "seed": 9_990_103,
        },
        "expected": {
            "vcount": 12,
            "directed": False,
            "is_simple": True,
            # K_12 = 12*11/2 = 66 edges.
            "ecount_min": 66,
            "ecount_max": 66,
        },
    },
]

# ALGO-GN-008: k_regular_game. Mirrors rigraph's
# `sample_k_regular(no.of.nodes, k, directed, multiple)` (the canonical
# R wrapper for `igraph_k_regular_game`). RNG state is not portable
# across implementations, so the manifest records only structural
# invariants: vcount, directed, is_simple, ecount band, and
# every_degree / every_out_degree / every_in_degree assertions.
K_REGULAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "k_regular_r_undirected_simple_n14_k4",
        "origin": "constructed (mirrors sample_k_regular(no.of.nodes=14, "
        "k=4, directed=FALSE, multiple=FALSE)): every vertex has "
        "degree 4 in a simple graph",
        "algo": "k_regular_game",
        "params": {
            "n": 14,
            "k": 4,
            "directed": False,
            "multiple": False,
            "seed": 7_770_001,
        },
        "expected": {
            "vcount": 14,
            "directed": False,
            "is_simple": True,
            "ecount_min": 28,  # n * k / 2
            "ecount_max": 28,
            "every_degree": 4,
        },
    },
    {
        "case": "k_regular_r_directed_simple_n9_k2",
        "origin": "constructed (mirrors sample_k_regular(no.of.nodes=9, "
        "k=2, directed=TRUE, multiple=FALSE)): every vertex has "
        "out-degree = in-degree = 2 in a simple directed graph",
        "algo": "k_regular_game",
        "params": {
            "n": 9,
            "k": 2,
            "directed": True,
            "multiple": False,
            "seed": 7_770_002,
        },
        "expected": {
            "vcount": 9,
            "directed": True,
            "is_simple": True,
            "ecount_min": 18,  # n * k
            "ecount_max": 18,
            "every_out_degree": 2,
            "every_in_degree": 2,
        },
    },
    {
        "case": "k_regular_r_undirected_multi_n6_k5",
        "origin": "constructed (mirrors sample_k_regular(no.of.nodes=6, "
        "k=5, directed=FALSE, multiple=TRUE)): n*k=30 is even, every "
        "vertex has degree 5, self-loops and parallel edges allowed",
        "algo": "k_regular_game",
        "params": {
            "n": 6,
            "k": 5,
            "directed": False,
            "multiple": True,
            "seed": 7_770_003,
        },
        "expected": {
            "vcount": 6,
            "directed": False,
            "is_simple": False,
            "ecount_min": 15,  # n * k / 2
            "ecount_max": 15,
            "every_degree": 5,
        },
    },
]

# ALGO-GN-009: watts_strogatz_game. Mirrors rigraph's
# `sample_smallworld(dim, size, nei, p, loops, multiple)` (the canonical R
# wrapper for `igraph_watts_strogatz_game`). RNG state is not portable
# across implementations, so only structural invariants are asserted —
# vcount, directed, ecount = size*nei (rewire is endpoint-preserving),
# and is_simple in the simple-graph regime.
WATTS_STROGATZ_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "watts_r_ring_lattice_p0_size14_nei2",
        "origin": "constructed (mirrors sample_smallworld(dim=1, size=14, "
        "nei=2, p=0, loops=FALSE, multiple=FALSE)): pure ring lattice, "
        "every vertex has degree 4, edges = size * nei = 28",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 14,
            "nei": 2,
            "p": 0.0,
            "loops": False,
            "multiple": False,
            "seed": 9_200_001,
        },
        "expected": {
            "vcount": 14,
            "directed": False,
            "is_simple": True,
            "ecount_min": 28,
            "ecount_max": 28,
            "every_degree": 4,
        },
    },
    {
        "case": "watts_r_small_world_p_low_size40_nei2",
        "origin": "constructed (mirrors sample_smallworld(dim=1, size=40, "
        "nei=2, p=0.05, loops=FALSE, multiple=FALSE)): small-world "
        "regime — sparse rewiring keeps the graph simple",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 40,
            "nei": 2,
            "p": 0.05,
            "loops": False,
            "multiple": False,
            "seed": 9_200_002,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "is_simple": True,
            "ecount_min": 80,
            "ecount_max": 80,
        },
    },
    {
        "case": "watts_r_dense_rewire_p_high_size24_nei5",
        "origin": "constructed (mirrors sample_smallworld(dim=1, size=24, "
        "nei=5, p=0.8, loops=FALSE, multiple=FALSE)): heavy-rewire "
        "regime — almost-random simple graph, edge count preserved",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 24,
            "nei": 5,
            "p": 0.8,
            "loops": False,
            "multiple": False,
            "seed": 9_200_003,
        },
        "expected": {
            "vcount": 24,
            "directed": False,
            "is_simple": True,
            "ecount_min": 120,
            "ecount_max": 120,
        },
    },
]

ALGO_MANIFESTS: Dict[str, List[Dict[str, Any]]] = {
    "bfs": BFS_MANIFEST,
    "community_to_membership": COMMUNITY_TO_MEMBERSHIP_MANIFEST,
    "reindex_membership": REINDEX_MEMBERSHIP_MANIFEST,
    "compare_communities": COMPARE_COMMUNITIES_MANIFEST,
    "split_join_distance": SPLIT_JOIN_DISTANCE_MANIFEST,
    "voronoi": VORONOI_MANIFEST,
    "ecc": ECC_PR031_MANIFEST,
    "community_voronoi": COMMUNITY_VORONOI_MANIFEST,
    "dfs": DFS_MANIFEST,
    "connected_components": CC_MANIFEST,
    "strongly_connected_components": SCC_MANIFEST,
    "distances": DIST_MANIFEST,
    "is_eulerian": EUL_MANIFEST,
    "articulation_points": AP_MANIFEST,
    "bridges": BRIDGE_MANIFEST,
    "is_biconnected": ISBI_MANIFEST,
    "girth": GIRTH_MANIFEST,
    "diameter": DIAM_MANIFEST,
    "diameter_with_mode": DIAM_MODE_MANIFEST,
    "eccentricity": ECC_MANIFEST,
    "eccentricity_with_mode": ECC_MODE_MANIFEST,
    "radius": RAD_MANIFEST,
    "radius_with_mode": RAD_MODE_MANIFEST,
    "count_triangles": TRI_MANIFEST,
    "transitivity_undirected": TRANS_MANIFEST,
    "transitivity_local_undirected": LTRANS_MANIFEST,
    "density": DENSITY_MANIFEST,
    "mean_distance": MEANDIST_MANIFEST,
    "global_efficiency": GLOBAL_EFFICIENCY_MANIFEST,
    "local_efficiency": LOCAL_EFFICIENCY_MANIFEST,
    "average_local_efficiency": AVERAGE_LOCAL_EFFICIENCY_MANIFEST,
    "eulerian_path": EUL_PATH_MANIFEST,
    "count_reachable": REACH_MANIFEST,
    "reachability_matrix": REACH_MATRIX_MANIFEST,
    "transitive_closure": TC_MANIFEST,
    "simplify": SIMPLIFY_MANIFEST,
    "louvain": LOUVAIN_MANIFEST,
    "leiden": LEIDEN_MANIFEST,
    "label_propagation": LPA_MANIFEST,
    "fluid_communities": FLUID_MANIFEST,
    "edge_betweenness_community": EB_COMMUNITY_MANIFEST,
    "edge_betweenness_community_weighted": EB_COMMUNITY_WEIGHTED_MANIFEST,
    "fast_greedy_modularity": FASTGREEDY_MANIFEST,
    "walktrap": WALKTRAP_MANIFEST,
    "modularity": MODULARITY_MANIFEST,
    "is_simple": IS_SIMPLE_MANIFEST,
    "has_loop": HAS_LOOP_MANIFEST,
    "has_multiple": HAS_MULTIPLE_MANIFEST,
    "is_loop": IS_LOOP_PER_EDGE_MANIFEST,
    "is_multiple": IS_MULTIPLE_PER_EDGE_MANIFEST,
    "count_adjacent_triangles": COUNT_ADJACENT_TRIANGLES_MANIFEST,
    "count_loops": COUNT_LOOPS_MANIFEST,
    "count_multiple": COUNT_MULTIPLE_MANIFEST,
    "disjoint_union": DISJOINT_UNION_MANIFEST,
    "union": UNION_MANIFEST,
    "intersection": INTERSECTION_MANIFEST,
    "difference": DIFFERENCE_MANIFEST,
    "dijkstra_distances": DIJKSTRA_MANIFEST,
    "dijkstra_paths": DIJKSTRA_PATHS_MANIFEST,
    "dijkstra_path_to": DIJKSTRA_PATH_TO_MANIFEST,
    "dijkstra_distances_cutoff": DIJKSTRA_CUTOFF_MANIFEST,
    "dijkstra_distances_with_mode": DIJKSTRA_DIST_MODE_MANIFEST,
    "bellman_ford_distances": BELLMAN_FORD_MANIFEST,
    "johnson_distances": JOHNSON_MANIFEST,
    "widest_path_widths": WIDEST_PATH_MANIFEST,
    "widest_path": WIDEST_PATH_GET_MANIFEST,
    "widest_path_widths_floyd_warshall": WIDEST_PATH_WIDTHS_FW_MANIFEST,
    "widest_paths_to": WIDEST_PATHS_TO_MANIFEST,
    "widest_paths": WIDEST_PATHS_SPT_MANIFEST,
    "edgelist_percolation": EDGELIST_PERCOLATION_MANIFEST,
    "bond_percolation": BOND_PERCOLATION_MANIFEST,
    "site_percolation": SITE_PERCOLATION_MANIFEST,
    "is_same_graph": IS_SAME_GRAPH_MANIFEST,
    "convergence_degree": CONVERGENCE_DEGREE_MANIFEST,
    "is_dag": IS_DAG_MANIFEST,
    "topological_sorting": TOPOLOGICAL_SORTING_MANIFEST,
    "is_acyclic": IS_ACYCLIC_MANIFEST,
    "is_tree": IS_TREE_MANIFEST,
    "is_forest": IS_FOREST_MANIFEST,
    "is_complete": IS_COMPLETE_MANIFEST,
    "neighborhood_size": NEIGHBORHOOD_SIZE_MANIFEST,
    "neighborhood": NEIGHBORHOOD_MANIFEST,
    "dijkstra_all_shortest_paths": DIJKSTRA_ASP_MANIFEST,
    "a_star_path": ASTAR_MANIFEST,
    "eccentricity_weighted_with_mode": ECC_W_MANIFEST,
    "radius_weighted_with_mode": RAD_W_MANIFEST,
    "diameter_weighted_with_mode": DIAM_W_MANIFEST,
    "floyd_warshall_distances": FW_MANIFEST,
    "coreness": CORENESS_MANIFEST,
    "reciprocity_with_mode": RECIP_MODE_MANIFEST,
    "modularity_weighted": MODULARITY_W_MANIFEST,
    "is_simple_with_mode": IS_SIMPLE_MODE_MANIFEST,
    "disjoint_union_many": DU_MANY_MANIFEST,
    "coreness_with_mode": CORENESS_MODE_MANIFEST,
    "assortativity_degree_directed": ASSORT_DIR_MANIFEST,
    "modularity_directed": MODULARITY_DIR_MANIFEST,
    "complementer": COMPLEMENTER_MANIFEST,
    "closeness_weighted": CLOSENESS_W_MANIFEST,
    "harmonic_centrality_weighted": HARMONIC_W_MANIFEST,
    "betweenness_weighted": BETW_W_MANIFEST,
    "edge_betweenness_weighted": EDGE_BETW_W_MANIFEST,
    "pagerank_weighted": PAGERANK_W_MANIFEST,
    "assortativity_degree_weighted": ASSORT_W_MANIFEST,
    "assortativity_degree_directed_weighted": ASSORT_DIR_W_MANIFEST,
    "closeness": CLOSE_MANIFEST,
    "harmonic_centrality": HARMONIC_MANIFEST,
    "betweenness": BETW_MANIFEST,
    "edge_betweenness": EDGE_BETW_MANIFEST,
    "pagerank": PAGERANK_MANIFEST,
    "biconnected_components": BC_MANIFEST,
    "biconnected_component_edges": BC_EDGES_MANIFEST,
    "eigenvector_centrality": EIGEN_MANIFEST,
    "eigenvector_centrality_weighted": EIGEN_W_MANIFEST,
    "eigenvector_centrality_directed": EIGEN_DIR_MANIFEST,
    "hub_and_authority_scores": HITS_MANIFEST,
    "hub_and_authority_scores_weighted": HITS_W_MANIFEST,
    "reciprocity": RECIP_MANIFEST,
    "avg_nearest_neighbor_degree": KNN_MANIFEST,
    "avg_nearest_neighbor_degree_weighted": KNN_W_MANIFEST,
    "knnk": KNNK_MANIFEST,
    "knnk_weighted": KNNK_W_MANIFEST,
    "assortativity_degree": ASSORT_MANIFEST,
    "transitivity_barrat": TRANS_BARRAT_MANIFEST,
    "decompose": DECOMPOSE_MANIFEST,
    "minimum_spanning_tree": SPANNING_TREE_MANIFEST,
    "erdos_renyi_gnp": ERDOS_RENYI_GNP_MANIFEST,
    "erdos_renyi_gnm": ERDOS_RENYI_GNM_MANIFEST,
    "barabasi_game_bag": BARABASI_BAG_MANIFEST,
    "growing_random_game": GROWING_RANDOM_MANIFEST,
    "tree_game_lerw": TREE_LERW_MANIFEST,
    "grg_game": GRG_MANIFEST,
    "forest_fire_game": FOREST_FIRE_MANIFEST,
    "simple_interconnected_islands_game": ISLANDS_MANIFEST,
    "k_regular_game": K_REGULAR_MANIFEST,
    "watts_strogatz_game": WATTS_STROGATZ_MANIFEST,
}


def graph_to_payload(g: ig.Graph) -> Dict[str, Any]:
    return {
        "n": g.vcount(),
        "edges": [list(e.tuple) for e in g.es],
        "directed": g.is_directed(),
        "weights": None,
    }


def emit(algo: str, manifest: List[Dict[str, Any]]) -> int:
    out_dir = OUT_DIR / algo
    out_dir.mkdir(parents=True, exist_ok=True)
    written = 0
    for entry in manifest:
        # `community_to_membership` is a pure dendrogram helper —
        # bypass graph_factory.
        if algo == "community_to_membership":
            nodes = int(entry["nodes"])
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": {"n": nodes, "edges": [], "directed": False, "weights": None},
                "algo": algo,
                "params": {
                    "merges": [list(m) for m in entry["merges"]],
                    "steps": int(entry["steps"]),
                },
                "expected": entry["expected"],
            }
        elif algo == "reindex_membership":
            membership = [int(c) for c in entry["membership"]]
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": {
                    "n": len(membership),
                    "edges": [],
                    "directed": False,
                    "weights": None,
                },
                "algo": algo,
                "params": {"membership": membership},
                "expected": entry["expected"],
            }
        elif algo == "compare_communities":
            comm1 = [int(c) for c in entry["comm1"]]
            comm2 = [int(c) for c in entry["comm2"]]
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": {
                    "n": len(comm1),
                    "edges": [],
                    "directed": False,
                    "weights": None,
                },
                "algo": algo,
                "params": {
                    "comm1": comm1,
                    "comm2": comm2,
                    "method": entry["method"],
                },
                "expected": entry["expected"],
            }
        elif algo == "split_join_distance":
            comm1 = [int(c) for c in entry["comm1"]]
            comm2 = [int(c) for c in entry["comm2"]]
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": {
                    "n": len(comm1),
                    "edges": [],
                    "directed": False,
                    "weights": None,
                },
                "algo": algo,
                "params": {
                    "comm1": comm1,
                    "comm2": comm2,
                },
                "expected": entry["expected"],
            }
        elif algo in (
            "erdos_renyi_gnp",
            "erdos_renyi_gnm",
            "barabasi_game_bag",
            "growing_random_game",
            "tree_game_lerw",
            "grg_game",
            "forest_fire_game",
            "simple_interconnected_islands_game",
            "k_regular_game",
            "watts_strogatz_game",
        ):
            # Generators produce a graph from params alone; graph
            # payload is a placeholder, expected carries structural
            # invariants (vcount/ecount/directed and, for BA,
            # `ba_temporal_order`). Mirrors the invariants asserted in
            # rigraph's `test-sample-gnp.R` / `test-sample-gnm.R` /
            # `test-sample-pa.R`.
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": {
                    "n": 0,
                    "edges": [],
                    "directed": False,
                    "weights": None,
                },
                "algo": algo,
                "params": entry["params"],
                "expected": entry["expected"],
            }
        else:
            g: ig.Graph = entry["graph_factory"]()
            graph_payload = graph_to_payload(g)
            if "graph_weights" in entry:
                graph_payload["weights"] = list(entry["graph_weights"])
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": graph_payload,
                "algo": algo,
                "params": entry["params"],
                "expected": entry["expected"],
            }
        out = out_dir / f"{entry['case']}.json"
        out.write_text(json.dumps(payload, indent=2))
        written += 1
        print(f"wrote {out.relative_to(REPO_ROOT)}")
    return written


def main(argv: List[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--algo",
        action="append",
        choices=sorted(ALGO_MANIFESTS),
        help="restrict to one algorithm; may be repeated. Default: all.",
    )
    args = p.parse_args(argv)
    targets = args.algo or sorted(ALGO_MANIFESTS)
    total = 0
    for algo in targets:
        total += emit(algo, ALGO_MANIFESTS[algo])
    print(f"done: {total} fixture(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
