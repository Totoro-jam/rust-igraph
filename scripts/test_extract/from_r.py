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

ALGO_MANIFESTS: Dict[str, List[Dict[str, Any]]] = {
    "bfs": BFS_MANIFEST,
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
    "eccentricity": ECC_MANIFEST,
    "radius": RAD_MANIFEST,
    "count_triangles": TRI_MANIFEST,
    "transitivity_undirected": TRANS_MANIFEST,
    "transitivity_local_undirected": LTRANS_MANIFEST,
    "density": DENSITY_MANIFEST,
    "mean_distance": MEANDIST_MANIFEST,
    "eulerian_path": EUL_PATH_MANIFEST,
    "count_reachable": REACH_MANIFEST,
    "reachability_matrix": REACH_MATRIX_MANIFEST,
    "transitive_closure": TC_MANIFEST,
    "simplify": SIMPLIFY_MANIFEST,
    "modularity": MODULARITY_MANIFEST,
    "is_simple": IS_SIMPLE_MANIFEST,
    "has_loop": HAS_LOOP_MANIFEST,
    "has_multiple": HAS_MULTIPLE_MANIFEST,
    "is_loop": IS_LOOP_PER_EDGE_MANIFEST,
    "is_multiple": IS_MULTIPLE_PER_EDGE_MANIFEST,
    "disjoint_union": DISJOINT_UNION_MANIFEST,
    "dijkstra_distances": DIJKSTRA_MANIFEST,
    "floyd_warshall_distances": FW_MANIFEST,
    "coreness": CORENESS_MANIFEST,
    "reciprocity_with_mode": RECIP_MODE_MANIFEST,
    "modularity_weighted": MODULARITY_W_MANIFEST,
    "is_simple_with_mode": IS_SIMPLE_MODE_MANIFEST,
    "complementer": COMPLEMENTER_MANIFEST,
    "closeness_weighted": CLOSENESS_W_MANIFEST,
    "harmonic_centrality_weighted": HARMONIC_W_MANIFEST,
    "betweenness_weighted": BETW_W_MANIFEST,
    "edge_betweenness_weighted": EDGE_BETW_W_MANIFEST,
    "pagerank_weighted": PAGERANK_W_MANIFEST,
    "assortativity_degree_weighted": ASSORT_W_MANIFEST,
    "assortativity_degree_weighted": ASSORT_W_MANIFEST,
    "closeness": CLOSE_MANIFEST,
    "harmonic_centrality": HARMONIC_MANIFEST,
    "betweenness": BETW_MANIFEST,
    "edge_betweenness": EDGE_BETW_MANIFEST,
    "pagerank": PAGERANK_MANIFEST,
    "biconnected_components": BC_MANIFEST,
    "eigenvector_centrality": EIGEN_MANIFEST,
    "reciprocity": RECIP_MANIFEST,
    "avg_nearest_neighbor_degree": KNN_MANIFEST,
    "assortativity_degree": ASSORT_MANIFEST,
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
