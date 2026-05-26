#!/usr/bin/env python3
"""Extract conformance fixtures from `references/python-igraph/tests/test_*.py`.

Phase 0 demo: hand-curated manifest covering BFS. The proper Phase-1 extractor
walks the AST of `unittest.TestCase` subclasses and harvests every
`assertEqual(g.<algo>(...), <expected>)` pair automatically.

Output: `tests/conformance/py/<algo>/<case>.json`

Usage:
    .venv/bin/python -m scripts.test_extract.from_py --algo bfs
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Callable, Dict, List

import igraph as ig

REPO_ROOT = Path(__file__).resolve().parents[2]
PY_TESTS_DIR = REPO_ROOT / "references/python-igraph/tests"
OUT_DIR = REPO_ROOT / "tests/conformance/py"


def _tree(n: int, children: int) -> ig.Graph:
    return ig.Graph.Tree(n=n, children=children, mode="undirected")


# Each entry mirrors one assertEqual from a python-igraph test method.
# Keep the python-igraph file:line in `origin` for traceability.
BFS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "test_iterators_testBFS_tree10_2",
        "origin": "test_iterators.py:IteratorTests.testBFS Tree(10,2) bfs(0)",
        "graph_factory": lambda: _tree(10, 2),
        "algo": "bfs",
        "params": {"root": 0},
        # Verbatim from upstream test:
        #   self.assertEqual(vs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
        "expected": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    },
]

DFS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "test_iterators_testDFS_tree10_2",
        "origin": "test_iterators.py:IteratorTests.testDFS Tree(10,2) dfs(0)",
        "graph_factory": lambda: _tree(10, 2),
        "algo": "dfs",
        "params": {"root": 0},
        # Verbatim from upstream test:
        #   self.assertEqual(vs, [0, 2, 6, 5, 1, 4, 9, 3, 8, 7])
        "expected": [0, 2, 6, 5, 1, 4, 9, 3, 8, 7],
    },
]

CC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "two_K3_components",
        # No verbatim test in test_decomposition.py — uses random graphs.
        # Hand-crafted but classic: two K3 cliques, weakly disconnected.
        # Verifies dense-id assignment in vertex-id order.
        "origin": "constructed: two disjoint K3 cliques; verified via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5)],
            directed=False,
        ),
        "algo": "connected_components",
        "params": {},
        "expected": {"membership": [0, 0, 0, 1, 1, 1], "count": 2},
    },
]

TRI_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "triangle_5_cycle_zero",
        # 5-cycle has zero triangles.
        "origin": "constructed: 5-cycle via python-igraph; 0 triangles",
        "graph_factory": lambda: ig.Graph.Ring(
            n=5, directed=False, mutual=False, circular=True
        ),
        "algo": "count_triangles",
        "params": {},
        "expected": 0,
    },
]

KNN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knn_K4",
        # K4: every vertex has 3 neighbours each of degree 3 → knn = [3,3,3,3].
        "origin": "constructed: K4 — knn = [3, 3, 3, 3]",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "avg_nearest_neighbor_degree",
        "params": {},
        "expected": [3.0, 3.0, 3.0, 3.0],
    },
]

KNN_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knn_weighted_py_3path_skewed",
        # 3-path 0-1-2 with weights (e0=1, e1=3). degrees [1, 2, 1].
        # Vertex 0: only neighbour is 1 (deg 2). knn_w[0] = (1*2)/1 = 2.
        # Vertex 1: incident to e0 (→0, deg 1, w=1) + e1 (→2, deg 1, w=3).
        #            sum = 1*1 + 3*1 = 4; strength = 1+3 = 4; knn_w[1] = 1.
        # Vertex 2: only neighbour is 1 (deg 2). knn_w[2] = (3*2)/3 = 2.
        "origin": "constructed: 3-path with weights (1,3); hand-checked knn_w = [2, 1, 2]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 3.0],
        "algo": "avg_nearest_neighbor_degree_weighted",
        "params": {},
        "expected": [2.0, 1.0, 2.0],
    },
]

KNNK_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knnk_py_K4",
        # K4: all knn = 3. knnk[0..1] = None (no deg 1, 2 vertices).
        # knnk[2] (deg 3) = 3.0.
        "origin": "constructed: K4; knnk = [None, None, 3.0]",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "knnk",
        "params": {},
        "expected": [None, None, 3.0],
    },
]

KNNK_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knnk_weighted_py_3path_skewed",
        # 3-path with weights (1, 3): knn_w = [2, 1, 2]; degrees [1, 2, 1].
        # knnk_w[0] (deg 1) = pooled over vertices 0 and 2.
        #   v0: sum = 1*2 = 2; str = 1. v2: sum = 3*2 = 6; str = 3.
        #   knnk_w[0] = (2 + 6) / (1 + 3) = 8/4 = 2.
        # knnk_w[1] (deg 2) = vertex 1 alone: sum = 4; str = 4 → 1.
        "origin": "constructed: weighted 3-path; knnk_w = [2.0, 1.0]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 3.0],
        "algo": "knnk_weighted",
        "params": {},
        "expected": [2.0, 1.0],
    },
]

DECOMPOSE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "decompose_py_three_components",
        # Three components: K3 on {0,1,2}, P3 on {3,4,5}, isolated {6}.
        # BFS-from-actstart on each component visits in identity order
        # (sorted neighbours). Remapped to per-component local 0..k-1.
        "origin": "constructed: K3 ∪ P3 ∪ isolated; hand-checked decompose",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5)],
            directed=False,
        ),
        "algo": "decompose",
        "params": {},
        "expected": [
            {
                "vcount": 3,
                "directed": False,
                "edges": [[0, 1], [0, 2], [1, 2]],
            },
            {
                "vcount": 3,
                "directed": False,
                "edges": [[0, 1], [1, 2]],
            },
            {
                "vcount": 1,
                "directed": False,
                "edges": [],
            },
        ],
    },
]

TRANS_BARRAT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_barrat_py_diamond_unit",
        # K4 minus edge (0,3) ("diamond"). Unit weights ⇒ Barrat reduces
        # to unweighted local clustering: vertex 0/3 deg 2, sees one
        # triangle → 1.0; vertex 1/2 deg 3, sees two triangles → 2/3.
        "origin": "constructed: K4 minus edge with unit weights",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "graph_weights": [1.0] * 5,
        "algo": "transitivity_barrat",
        "params": {},
        "expected": [1.0, 2.0 / 3.0, 2.0 / 3.0, 1.0],
    },
]

RECIP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reciprocity_mutual_pair",
        # Two mutual edges 0<->1: reciprocity 1.0.
        "origin": "constructed: mutual pair — reciprocity 1.0",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (1, 0)], directed=True
        ),
        "algo": "reciprocity",
        "params": {},
        "expected": 1.0,
    },
]

EIGEN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eigenvector_star_4",
        # 4-star: centre 1.0, leaves 1/sqrt(3).
        "origin": "constructed: 4-star; centre 1.0, leaves 1/sqrt(3)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "eigenvector_centrality",
        "params": {},
        "expected": [
            1.0,
            0.5773502691896258,
            0.5773502691896258,
            0.5773502691896258,
        ],
    },
]

EIGEN_W_MANIFEST: List[Dict[str, Any]] = [
    {
        # Triangle with unit weights — weighted adjacency = unweighted
        # adjacency; closed-form vec=[1,1,1], λ=2.
        "case": "eigenvector_w_py_triangle_unit",
        "origin": "constructed: triangle with unit weights; vec=[1,1,1], λ=2",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0],
        "algo": "eigenvector_centrality_weighted",
        "params": {},
        "expected": {
            "vector": [1.0, 1.0, 1.0],
            "eigenvalue": 2.0,
        },
    },
]

EIGEN_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        # Directed K_5 (complete digraph, no loops): every vertex has
        # equal centrality 1.0; eigenvalue = n-1 = 4. Mode = OUT.
        "case": "eigenvector_dir_py_k5_directed_out",
        "origin": "constructed: directed K5 (no loops); vec=[1,1,1,1,1], λ=4",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(u, v) for u in range(5) for v in range(5) if u != v],
            directed=True,
        ),
        "algo": "eigenvector_centrality_directed",
        "params": {"mode": "out"},
        "expected": {
            "vector": [1.0, 1.0, 1.0, 1.0, 1.0],
            "eigenvalue": 4.0,
        },
    },
]

HITS_W_MANIFEST: List[Dict[str, Any]] = [
    {
        # Two hubs into one authority with weights (2, 3). Closed form:
        # W·W^T (top-left 2x2 = [[4,6],[6,9]]) has λ=13, principal
        # eigenvector (2/3, 1). Authority = W^T·hub = (0,0,13/3) →
        # max-norm (0,0,1). Doctest-friendly hand computation.
        "case": "hits_w_py_two_hubs_one_authority_weighted",
        "origin": "constructed: 0→2 (w=2), 1→2 (w=3); λ=13 closed form",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 2), (1, 2)], directed=True
        ),
        "graph_weights": [2.0, 3.0],
        "algo": "hub_and_authority_scores_weighted",
        "params": {},
        "expected": {
            "hub": [2.0 / 3.0, 1.0, 0.0],
            "authority": [0.0, 0.0, 1.0],
            "eigenvalue": 13.0,
        },
    },
]

HITS_MANIFEST: List[Dict[str, Any]] = [
    {
        # python-igraph: Graph.hub_score / authority_score on a 2x2
        # bipartite hubs→authorities pattern. With max-abs scaling, the
        # two source vertices are pure hubs and the two sink vertices
        # are pure authorities. The largest A·Aᵀ eigenvalue is 4
        # (block-diagonal 2x2 rank-1 matrix with all-ones on the hub
        # side).
        "case": "hits_py_bipartite_2x2",
        "origin": "constructed: 0,1 → 2,3 — pure hub/authority partition",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 2), (0, 3), (1, 2), (1, 3)], directed=True
        ),
        "algo": "hub_and_authority_scores",
        "params": {},
        "expected": {
            "hub": [1.0, 1.0, 0.0, 0.0],
            "authority": [0.0, 0.0, 1.0, 1.0],
            "eigenvalue": 4.0,
        },
    },
    {
        # Directed triangle: every vertex is symmetrically a hub and an
        # authority of the same magnitude. Aligns with python-igraph's
        # hub_score/authority_score reporting (max-norm convention).
        "case": "hits_py_directed_triangle_uniform",
        "origin": "constructed: 0→1→2→0 — uniform hub & authority, A·Aᵀ eigenvalue 1",
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

BC_EDGES_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "biconnected_component_edges_py_triangle_plus_pendant",
        # Triangle 0-1-2 + pendant 0-3: two components.
        # Triangle component edges: {0-1, 1-2, 0-2}; pendant: {0-3}.
        "origin": "constructed: triangle 0-1-2 + pendant 0-3 — CC-012 partition",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 0), (0, 3)], directed=False
        ),
        "algo": "biconnected_component_edges",
        "params": {},
        "expected": sorted(
            [
                sorted([[0, 1], [1, 2], [0, 2]]),
                [[0, 3]],
            ]
        ),
    },
]

BC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "biconnected_components_star_4",
        # 4-star: each leaf-edge is its own biconnected component.
        "origin": "constructed: 4-star; 3 components, 1 articulation point",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "biconnected_components",
        "params": {},
        "expected": {
            "count": 3,
            "components": [[0, 1], [0, 2], [0, 3]],
            "articulation_points": [0],
        },
    },
]

PAGERANK_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "pagerank_triangle",
        # Triangle: uniform 1/3.
        "origin": "constructed: triangle; uniform PageRank 1/3",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "pagerank",
        "params": {},
        "expected": [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
    },
]

EDGE_BETW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_betweenness_star_4",
        # Star centre: each edge serves 3 pairs (centre+leaf, plus 2 leaf-leaf
        # pairs that traverse this edge).
        "origin": "constructed: 4-star; each edge betweenness = 3.0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "edge_betweenness",
        "params": {},
        "expected": [3.0, 3.0, 3.0],
    },
]

BETW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "betweenness_star_4",
        # Star: centre 3.0 (sits on all 3 leaf-leaf paths), leaves 0.
        "origin": "constructed: 4-star; centre betweenness 3.0, leaves 0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "betweenness",
        "params": {},
        "expected": [3.0, 0.0, 0.0, 0.0],
    },
]

HARMONIC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "harmonic_star_4",
        # 4-star: centre 1.0, leaves 2/3.
        "origin": "constructed: 4-vertex star; harmonic centre 1.0, leaves 2/3",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "harmonic_centrality",
        "params": {},
        "expected": [1.0, 2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0],
    },
]

CLOSE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "closeness_star_4",
        # Star with centre 0: centre 1.0, leaves 0.6.
        "origin": "constructed: 4-vertex star; centre 1.0, leaves 0.6",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "closeness",
        "params": {},
        "expected": [1.0, 0.6, 0.6, 0.6],
    },
]

ASSORT_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assort_w_py_path_4_unit_weights",
        # Unit-weight 4-path collapses to unweighted assortativity_degree
        # = -0.5 (per python-igraph oracle; matches PR-006-style formula).
        "origin": "constructed: 4-path with unit weights → -0.5 (python-igraph)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0],
        "algo": "assortativity_degree_weighted",
        "params": {},
        "expected": -0.500000000000003,
    },
]

PAGERANK_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "pagerank_w_py_heavy_edge_concentrates",
        # Directed 0→1@100, 0→2@0.01: vertex 1 gets ~all of 0's flow.
        # Computed via python-igraph 0.11 (ARPACK) to f64 precision.
        "origin": "constructed: directed 2-out + huge weight asymmetry",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2)], directed=True
        ),
        "graph_weights": [100.0, 0.01],
        "algo": "pagerank_weighted",
        "params": {},
        "expected": [
            0.2597402597402597,
            0.48049740480497405,
            0.2597623354547662,
        ],
    },
]

EDGE_BETW_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_betw_w_py_triangle_chord_swap",
        # Triangle (0,1)-(1,2)-(0,2) with weights (1, 1, 5): the chord
        # 0-2 is too expensive, so 0→1→2 wins. Edge (0,2) gets 0,
        # the two legs each get 2.0.
        "origin": "constructed: triangle with heavy chord; chord betweenness 0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 5.0],
        "algo": "edge_betweenness_weighted",
        "params": {},
        "expected": {
            "edges": [[0, 1], [1, 2], [0, 2]],
            "values": [2.0, 2.0, 0.0],
        },
    },
]

BETW_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "betw_w_py_path5_unit_weights",
        # Mirrors python-igraph's betweenness smoke test: 5-path with
        # unit weights collapses exactly to the unweighted expected
        # values [0, 3, 4, 3, 0].
        "origin": "constructed: 5-path with unit weights matches PR-008",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0, 1.0],
        "algo": "betweenness_weighted",
        "params": {},
        "expected": [0.0, 3.0, 4.0, 3.0, 0.0],
    },
]

HARMONIC_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "harmonic_w_py_directed_chain_with_shortcut",
        # Directed 0→1→2→3, plus 0→3 weight 5; shortest path 0→3 = 3
        # (via chain). Mirrors the dijkstra py fixture.
        "origin": "constructed: directed 4-vertex chain with shortcut",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (0, 3)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "harmonic_centrality_weighted",
        "params": {},
        "expected": [
            0.611111111111111,
            0.5,
            0.3333333333333333,
            0.0,
        ],
    },
]

CLOSENESS_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "closeness_weighted_py_directed_path",
        # Directed 0->1 (w=2.0), 1->2 (w=0.5).
        # 0 reaches {1@2.0, 2@2.5} → 2/4.5 = 4/9 ≈ 0.4444...
        # 1 reaches {2@0.5} → 1/0.5 = 2.0; 2 isolated.
        "origin": "constructed: directed path with weights (2.0, 0.5); "
        "0→1 dist 2, 0→2 dist 2.5",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "graph_weights": [2.0, 0.5],
        "algo": "closeness_weighted",
        "params": {},
        "expected": [0.4444444444444444, 2.0, None],
    },
]

COMPLEMENTER_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "complementer_py_isolated_with_loops",
        # 3 isolated vertices, loops=True → complete graph + 3 self-loops.
        "origin": "constructed: 3 isolated vertices; complementer(loops=True) is K3 + self-loops",
        "graph_factory": lambda: ig.Graph(n=3, edges=[], directed=False),
        "algo": "complementer",
        "params": {"loops": True},
        "expected": {
            "vcount": 3,
            "directed": False,
            "edges": [[0, 0], [0, 1], [0, 2], [1, 1], [1, 2], [2, 2]],
        },
    },
]

DIJKSTRA_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_py_directed_path_with_shortcut",
        # Mirrors python-igraph's `Graph.distances(weights=...)` smoke
        # test: directed 4-vertex graph with a long direct edge that
        # gets shortcut by the chain.
        "origin": "constructed: directed 4-vertex chain with shortcut "
        "0->3 (5.0) vs 0->1->2->3 (3.0)",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (0, 3)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "dijkstra_distances",
        "params": {"source": 0},
        "expected": [0.0, 1.0, 2.0, 3.0],
    },
]

# ALGO-SP-001b: paths variant — only `distances` checked.
DIJKSTRA_PATHS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_paths_py_directed_chain_shortcut",
        "origin": "constructed: directed 4-vertex chain with shortcut "
        "0->3 (5.0) vs 0->1->2->3 (3.0)",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (0, 3)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "dijkstra_paths",
        "params": {"source": 0},
        "expected": {"distances": [0.0, 1.0, 2.0, 3.0]},
    },
]

# ALGO-SP-001b: source-to-target convenience.
DIJKSTRA_PATH_TO_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_path_to_py_undirected_triangle_shortcut",
        # Undirected triangle with weights [1, 4, 2]: best path 0->1->2.
        "origin": "constructed: triangle (1,4,2) with shortcut via vertex 1",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "dijkstra_path_to",
        "params": {"source": 0, "target": 2},
        "expected": {"vertices": [0, 1, 2], "edges": [0, 2]},
    },
]

# ALGO-SP-001b: cutoff variant.
DIJKSTRA_CUTOFF_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_cutoff_py_directed_chain_cutoff_2",
        # Directed chain with cutoff 2.0: vertex 3 (distance 3) masked
        # to None; the heavy 0->3 edge alone with weight 5.0 is also
        # past the cutoff.
        "origin": "constructed: directed 4-vertex chain with cutoff=2.0",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (0, 3)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "dijkstra_distances_cutoff",
        "params": {"source": 0, "cutoff": 2.0},
        "expected": [0.0, 1.0, 2.0, None],
    },
]

# ALGO-PR-022: is_acyclic. python-igraph does not expose this;
# hand-computed expected values.
IS_ACYCLIC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_acyclic_py_undirected_triangle_false",
        "origin": "constructed: undirected triangle — cycle, not acyclic",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "is_acyclic",
        "params": {},
        "expected": False,
    },
    {
        "case": "is_acyclic_py_undirected_parallel_edge_false",
        # Two parallel undirected edges form a 2-cycle.
        "origin": "constructed: parallel undirected edges — 2-cycle, not acyclic",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1)], directed=False
        ),
        "algo": "is_acyclic",
        "params": {},
        "expected": False,
    },
]

# ALGO-PR-023: is_tree. python-igraph exposes
# `Graph.is_tree(mode='out'/'in'/'all')` returning a bool.
IS_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_tree_py_in_arborescence_true",
        # 1→0, 2→0, 3→1: in-tree rooted at 0 (every edge points TO root).
        "origin": "constructed: in-arborescence rooted at 0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(1, 0), (2, 0), (3, 1)], directed=True
        ),
        "algo": "is_tree",
        "params": {"mode": "in"},
        "expected": True,
    },
    {
        "case": "is_tree_py_v_pattern_not_out_tree_false",
        # 0→2, 1→2: vertex 2 has in-degree 2 — not an out-tree.
        "origin": "constructed: V-pattern to centre — not an out-tree",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 2), (1, 2)], directed=True
        ),
        "algo": "is_tree",
        "params": {"mode": "out"},
        "expected": False,
    },
]

# ALGO-PR-024: is_forest. python-igraph does NOT expose
# `Graph.is_forest`, so the oracle replicates the C contract
# inline. The fixtures here mirror common shapes the upstream
# Python tests exercise (small directed/undirected forests and
# negative cases).
IS_FOREST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_forest_py_directed_in_two_anti_arborescences_true",
        # 1→0, 3→2: every edge points to a sink — two in-trees.
        "origin": "constructed: 1→0 ⊔ 3→2 — 2 in-arborescences",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(1, 0), (3, 2)], directed=True
        ),
        "algo": "is_forest",
        "params": {"mode": "in"},
        "expected": {"is_forest": True, "roots": [0, 2]},
    },
    {
        "case": "is_forest_py_undirected_self_loop_false",
        # Self-loop on vertex 0; rest is forest.
        "origin": "constructed: self-loop on 0 + edge 1-2 — self-loop = cycle",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 0), (1, 2)], directed=False
        ),
        "algo": "is_forest",
        "params": {"mode": "all"},
        "expected": {"is_forest": False, "roots": []},
    },
]

# ALGO-PR-016: is_complete. python-igraph exposes
# `Graph.is_complete()` natively and returns True for the null
# and singleton graphs.
IS_COMPLETE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_complete_py_singleton_true",
        "origin": "constructed: 1-vertex graph — vacuously complete",
        "graph_factory": lambda: ig.Graph(n=1, edges=[], directed=False),
        "algo": "is_complete",
        "params": {},
        "expected": True,
    },
    {
        "case": "is_complete_py_k3_with_self_loop_true",
        # K_3 plus a self-loop at vertex 0 — slow path: ecount > target
        # but every vertex still sees both other vertices.
        "origin": "constructed: K_3 + self-loop — slow path returns true",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2), (0, 0)], directed=False
        ),
        "algo": "is_complete",
        "params": {},
        "expected": True,
    },
]

# ALGO-PR-027: neighborhood_size. python-igraph exposes
# `Graph.neighborhood_size(vertices=None, order=1, mode='all', mindist=0)`.
# Fixtures from tests/test_structural.py:testNeighborhoodSize.
NEIGHBORHOOD_SIZE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "neighborhood_size_py_ring10_order_1",
        "origin": "tests/test_structural.py testNeighborhoodSize — Ring(10, circular=False) order=1",
        "graph_factory": lambda: ig.Graph.Ring(n=10, circular=False),
        "algo": "neighborhood_size",
        "params": {"order": 1, "mode": "all", "mindist": 0},
        "expected": [2, 3, 3, 3, 3, 3, 3, 3, 3, 2],
    },
    {
        "case": "neighborhood_size_py_ring10_order_3_mindist_2",
        "origin": "tests/test_structural.py testNeighborhoodSize — Ring(10), order=3 mindist=2",
        "graph_factory": lambda: ig.Graph.Ring(n=10, circular=False),
        "algo": "neighborhood_size",
        "params": {"order": 3, "mode": "all", "mindist": 2},
        "expected": [2, 2, 3, 4, 4, 4, 4, 3, 2, 2],
    },
]

# ALGO-PR-027b: neighborhood (vertex lists). Fixtures from
# tests/test_structural.py:testNeighborhood. All expected lists are
# sorted (matches `list(map(sorted, g.neighborhood()))` in the upstream
# test).
NEIGHBORHOOD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "neighborhood_py_ring10_order_1",
        "origin": "tests/test_structural.py testNeighborhood — Ring(10, circular=False) order=1",
        "graph_factory": lambda: ig.Graph.Ring(n=10, circular=False),
        "algo": "neighborhood",
        "params": {"order": 1, "mode": "all", "mindist": 0},
        "expected": [
            [0, 1],
            [0, 1, 2],
            [1, 2, 3],
            [2, 3, 4],
            [3, 4, 5],
            [4, 5, 6],
            [5, 6, 7],
            [6, 7, 8],
            [7, 8, 9],
            [8, 9],
        ],
    },
    {
        "case": "neighborhood_py_ring10_order_3",
        "origin": "tests/test_structural.py testNeighborhood — Ring(10), order=3",
        "graph_factory": lambda: ig.Graph.Ring(n=10, circular=False),
        "algo": "neighborhood",
        "params": {"order": 3, "mode": "all", "mindist": 0},
        "expected": [
            [0, 1, 2, 3],
            [0, 1, 2, 3, 4],
            [0, 1, 2, 3, 4, 5],
            [0, 1, 2, 3, 4, 5, 6],
            [1, 2, 3, 4, 5, 6, 7],
            [2, 3, 4, 5, 6, 7, 8],
            [3, 4, 5, 6, 7, 8, 9],
            [4, 5, 6, 7, 8, 9],
            [5, 6, 7, 8, 9],
            [6, 7, 8, 9],
        ],
    },
]

# ALGO-PR-021: topological_sorting. python-igraph exposes
# `Graph.topological_sorting(mode='OUT'/'IN'/'ALL')`.
TOPOLOGICAL_SORTING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "topo_sort_py_self_loop_does_not_block",
        # Self-loop on vertex 0 + 0 → 1 — self-loops ignored, order = [0, 1].
        "origin": "constructed: self-loop tolerated by topological sort",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 0), (0, 1)], directed=True
        ),
        "algo": "topological_sorting",
        "params": {"mode": "out"},
        "expected": [0, 1],
    },
    {
        "case": "topo_sort_py_long_chain_unique_order",
        # 0 → 1 → 2 → 3 → 4: only one valid topological order; safe
        # to compare element-wise across implementations.
        "origin": "constructed: directed P5 — unique OUT topological order",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=True
        ),
        "algo": "topological_sorting",
        "params": {"mode": "out"},
        "expected": [0, 1, 2, 3, 4],
    },
]

# ALGO-PR-020: is_dag. python-igraph's `Graph.is_dag()` returns
# True/False directly.
IS_DAG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_dag_py_diamond_dag_true",
        # 0→1, 0→2, 1→3, 2→3 — diamond, no cycles.
        "origin": "constructed: directed diamond — DAG",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 3), (2, 3)], directed=True
        ),
        "algo": "is_dag",
        "params": {},
        "expected": True,
    },
    {
        "case": "is_dag_py_self_loop_false",
        # Self-loop on 0 plus an unrelated edge — vertex with a
        # self-loop cannot be topologically ordered.
        "origin": "constructed: self-loop disqualifies DAG",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 0), (0, 1)], directed=True
        ),
        "algo": "is_dag",
        "params": {},
        "expected": False,
    },
]

# ALGO-PR-028: convergence_degree. python-igraph exposes
# `Graph.convergence_degree()` returning a per-edge list. Edges that
# lie on no shortest path produce NaN; we encode NaN as JSON `null`
# in the expected vector.
CONVERGENCE_DEGREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "convergence_degree_py_undirected_triangle",
        # K_3 — every edge is symmetric ⇒ all zeros.
        "origin": "constructed: K_3 — symmetric, expect all zeros",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "algo": "convergence_degree",
        "params": {},
        "expected": [0.0, 0.0, 0.0],
    },
    {
        "case": "convergence_degree_py_directed_cycle_c3",
        # Directed 3-cycle — each edge sees one source, one sink ⇒ 0.
        "origin": "constructed: directed C_3 — balanced ⇒ all zeros",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "convergence_degree",
        "params": {},
        "expected": [0.0, 0.0, 0.0],
    },
]

# ALGO-CORE-001e: is_same_graph (structural equality). python-igraph
# does not expose this predicate; hand-computed expected values.
IS_SAME_GRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_same_graph_py_self_equal",
        # A graph is the same as itself.
        "origin": "constructed: triangle compared to itself ⇒ same",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "algo": "is_same_graph",
        "params": {
            "other": {
                "n": 3,
                "edges": [[0, 1], [0, 2], [1, 2]],
                "directed": False,
            }
        },
        "expected": True,
    },
    {
        "case": "is_same_graph_py_parallel_edge_multiplicity_matters",
        # {0-1, 0-1} vs {0-1}: ecount differs ⇒ not same.
        "origin": "constructed: parallel multiplicity differs ⇒ not same",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1)], directed=False
        ),
        "algo": "is_same_graph",
        "params": {
            "other": {
                "n": 2,
                "edges": [[0, 1]],
                "directed": False,
            }
        },
        "expected": False,
    },
]

# ALGO-CC-032: Site percolation. python-igraph does not bind this;
# hand-computed expected values.
SITE_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "site_perc_py_two_components_then_isolated",
        # Two disjoint edges (0,1) (2,3) plus isolated vertex 4.
        # Activate order: [0, 1, 2, 3, 4]
        "origin": "constructed: two pairs + isolated vertex",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (2, 3)], directed=False
        ),
        "algo": "site_percolation",
        "params": {"vertex_order": [0, 1, 2, 3, 4]},
        "expected": {
            "giant_size": [1, 2, 2, 2, 2],
            "edge_count": [0, 1, 1, 2, 2],
        },
    },
    {
        "case": "site_perc_py_star_centre_last",
        # Star around 0: edges (0,1), (0,2), (0,3). Activate leaves
        # first (1, 2, 3) then centre (0): leaves stay isolated until
        # 0 joins, then 3 edges burst.
        "origin": "constructed: star, centre activated last",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "site_percolation",
        "params": {"vertex_order": [1, 2, 3, 0]},
        "expected": {
            "giant_size": [1, 1, 1, 4],
            "edge_count": [0, 0, 0, 3],
        },
    },
]

# ALGO-CC-031: Bond percolation. python-igraph does not bind this;
# hand-computed expected values resolved from edge ids.
BOND_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bond_perc_py_star_around_center",
        # Star around vertex 0; add edges in id order.
        "origin": "constructed: star graph, natural id order",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "bond_percolation",
        "params": {"edge_order": [0, 1, 2]},
        "expected": {
            "giant_size": [2, 3, 4],
            "vertex_count": [2, 3, 4],
        },
    },
    {
        "case": "bond_perc_py_two_components_then_bridge",
        # Adding edges in the order [pair, pair, bridge].
        # First pair {0,1}, second pair {2,3}, then (1,2) bridges them.
        "origin": "constructed: two pairs joined by a bridge edge",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3), (1, 2)], directed=False
        ),
        "algo": "bond_percolation",
        "params": {"edge_order": [0, 1, 2]},
        "expected": {
            "giant_size": [2, 2, 4],
            "vertex_count": [2, 4, 4],
        },
    },
]

# ALGO-CC-030: Edge-list percolation. python-igraph does not bind
# this; hand-computed expected values.
EDGELIST_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edgelist_perc_py_star_around_center",
        # Star edges 0-1, 0-2, 0-3 → giant grows 2, 3, 4.
        "origin": "constructed: star around vertex 0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "edgelist_percolation",
        "params": {},
        "expected": {
            "giant_size": [2, 3, 4],
            "vertex_count": [2, 3, 4],
        },
    },
    {
        "case": "edgelist_perc_py_triangle_then_bridge_to_pair",
        # Build triangle {0,1,2}, separate pair {3,4}, then bridge.
        # Final state: one component of 5.
        "origin": "constructed: triangle + isolated pair + bridge",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (0, 2), (3, 4), (2, 3)], directed=False
        ),
        "algo": "edgelist_percolation",
        "params": {},
        "expected": {
            "giant_size": [2, 3, 3, 3, 5],
            "vertex_count": [2, 3, 3, 5, 5],
        },
    },
]

# ALGO-SP-014: Single-source widest-paths SPT struct (widths +
# parents + inbound_edges). Hand-computed; source's width null.
WIDEST_PATHS_SPT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_paths_py_directed_chain",
        # Directed 0→1→2→3 weights (5, 3, 4); widest from 0: chain
        # bottlenecks 5, 3, 3. Each vertex reached by the unique chain.
        "origin": "constructed: directed P4 (5,3,4) — unique SPT",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "graph_weights": [5.0, 3.0, 4.0],
        "algo": "widest_paths",
        "params": {"source": 0},
        "expected": {
            "widths": [None, 5.0, 3.0, 3.0],
            "parents": [None, 0, 1, 2],
            "inbound_edges": [None, 0, 1, 2],
        },
    },
    {
        "case": "widest_paths_py_chain_with_bottleneck",
        # 0-1-2-3 with weights 5, 1, 3; widths from 0 = 5, 1, 1.
        # Each vertex reached by the unique chain.
        "origin": "constructed: P4 (5,1,3) — bottleneck shrinks at edge 1",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [5.0, 1.0, 3.0],
        "algo": "widest_paths",
        "params": {"source": 0},
        "expected": {
            "widths": [None, 5.0, 1.0, 1.0],
            "parents": [None, 0, 1, 2],
            "inbound_edges": [None, 0, 1, 2],
        },
    },
]

# ALGO-SP-013: Multi-target widest paths.
WIDEST_PATHS_TO_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_paths_to_py_self_target_plus_normal",
        # Targets [from, neighbor]: first is trivial, second is a single edge.
        "origin": "constructed: P3 (5,3); targets include source itself",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [5.0, 3.0],
        "algo": "widest_paths_to",
        "params": {"from": 1, "targets": [1, 0]},
        "expected": [
            {"vertices": [1], "edges": []},
            {"vertices": [1, 0], "edges": [0]},
        ],
    },
    {
        "case": "widest_paths_to_py_directed_chain",
        # Directed 0 → 1 → 2 → 3, targets all three. OUT mode by default.
        "origin": "constructed: directed P4 (5,3,4) targets {1,2,3}",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "graph_weights": [5.0, 3.0, 4.0],
        "algo": "widest_paths_to",
        "params": {"from": 0, "targets": [1, 2, 3]},
        "expected": [
            {"vertices": [0, 1], "edges": [0]},
            {"vertices": [0, 1, 2], "edges": [0, 1]},
            {"vertices": [0, 1, 2, 3], "edges": [0, 1, 2]},
        ],
    },
]

# ALGO-SP-012: Floyd-Warshall-based all-pairs widest widths matrix.
# Hand-computed expected values; diagonal is +∞ by convention,
# encoded as null in fixtures.
WIDEST_PATH_WIDTHS_FW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_fw_py_undirected_chain",
        # Path 0-1-2-3 weights (5, 1, 3). All-pairs bottlenecks:
        # 0↔1=5; 0↔2=min(5,1)=1; 0↔3=min(5,1,3)=1; 1↔2=1; 1↔3=1; 2↔3=3.
        "origin": "constructed: undirected P4 (5,1,3)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [5.0, 1.0, 3.0],
        "algo": "widest_path_widths_floyd_warshall",
        "params": {},
        "expected": [
            [None, 5.0, 1.0, 1.0],
            [5.0, None, 1.0, 1.0],
            [1.0, 1.0, None, 3.0],
            [1.0, 1.0, 3.0, None],
        ],
    },
    {
        "case": "widest_fw_py_parallel_edges",
        # Two vertices, 3 parallel edges (1, 5, 3). Widest = 5.
        "origin": "constructed: 3 parallel edges (1, 5, 3)",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1), (0, 1)], directed=False
        ),
        "graph_weights": [1.0, 5.0, 3.0],
        "algo": "widest_path_widths_floyd_warshall",
        "params": {},
        "expected": [
            [None, 5.0],
            [5.0, None],
        ],
    },
]

# ALGO-SP-011: Widest path (single source-to-target). python-igraph
# does not bind this either; hand-computed expected values.
WIDEST_PATH_GET_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_get_py_self_target_trivial",
        # Source equals target: trivial zero-edge path.
        "origin": "constructed: self-target path is single vertex, no edges",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 1.0],
        "algo": "widest_path",
        "params": {"from": 1, "to": 1},
        "expected": {"vertices": [1], "edges": []},
    },
    {
        "case": "widest_get_py_directed_chain_out",
        # Directed 0 → 1 → 2 with weights 5, 3 in OUT mode.
        "origin": "constructed: directed P3 (5, 3) OUT — chain is unique path",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "graph_weights": [5.0, 3.0],
        "algo": "widest_path",
        "params": {"from": 0, "to": 2},
        "expected": {"vertices": [0, 1, 2], "edges": [0, 1]},
    },
]

# ALGO-SP-010: Widest-path widths. python-igraph does not bind the
# C `igraph_widest_path_widths_dijkstra`, so these expected values
# are hand-computed (same shape as our Rust API, source position null).
WIDEST_PATH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_py_unreachable_components",
        "origin": "constructed: 4 vertices, 2 disjoint edges; w[2]=w[3]=None",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [3.0, 7.0],
        "algo": "widest_path_widths",
        "params": {"source": 0},
        "expected": [None, 3.0, None, None],
    },
    {
        "case": "widest_py_negative_finite_weight",
        # A negative-but-finite weight is a valid bottleneck — only -inf
        # is the ignore sentinel. Source 0 → 1 (-1) → 2 (5):
        # widest = min(-1, 5) = -1.
        "origin": "constructed: chain with one negative-finite weight",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [-1.0, 5.0],
        "algo": "widest_path_widths",
        "params": {"source": 0},
        "expected": [None, -1.0, -1.0],
    },
]

# ALGO-SP-003: Johnson all-pairs distances. python-igraph's
# Graph.distances dispatches to Johnson when called over all pairs
# with negative weights; we encode the result as a full Vec<Vec<...>>.
JOHNSON_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "johnson_py_directed_diamond_negative_edge",
        "origin": "constructed: directed diamond with negative edge 1→3",
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
        "case": "johnson_py_undirected_positive_fast_path",
        "origin": "constructed: undirected triangle (1,4,2) — Johnson fast path",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "johnson_distances",
        "params": {},
        "expected": [
            [0.0, 1.0, 3.0],
            [1.0, 0.0, 2.0],
            [3.0, 2.0, 0.0],
        ],
    },
]

# ALGO-SP-002: Bellman-Ford single-source distances. python-igraph
# auto-dispatches Graph.distances to BF when weights contain negatives.
BELLMAN_FORD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bellman_ford_py_negative_edge_directed_diamond",
        # Same setup as the C fixture: 0→1 (3), 0→2 (1), 1→3 (-2), 2→3 (4).
        "origin": "constructed: directed diamond with negative edge 1→3",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 3), (2, 3)], directed=True
        ),
        "graph_weights": [3.0, 1.0, -2.0, 4.0],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 3.0, 1.0, 1.0],
    },
    {
        "case": "bellman_ford_py_undirected_positive_weights",
        "origin": "constructed: undirected triangle (1,4,2)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 1.0, 3.0],
    },
    {
        "case": "bellman_ford_py_unreachable_with_negative",
        # Two components: source's component has a positive edge,
        # other component has a negative edge that BF doesn't need
        # to touch.
        "origin": "constructed: two components, source in first",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [2.0, -1.0],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 2.0, None, None],
    },
]

# ALGO-SP-001c: mode-aware distances. ALL-mode treats directed graph
# as undirected.
DIJKSTRA_DIST_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_dist_mode_py_directed_path_all",
        "origin": "constructed: directed P3 (1,2), ALL mode = undirected projection",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "graph_weights": [1.0, 2.0],
        "algo": "dijkstra_distances_with_mode",
        "params": {"source": 0, "mode": "all"},
        "expected": [0.0, 1.0, 3.0],
    },
]

# ALGO-SP-001c: all-shortest-paths.
DIJKSTRA_ASP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_asp_py_unique_chain",
        # Triangle with shortcut weights (1,4,2): one geodesic to each
        # vertex (the 0→1→2 path beats the direct 0→2 edge).
        "origin": "constructed: triangle (1,4,2) with unique shortcut",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "dijkstra_all_shortest_paths",
        "params": {"source": 0, "mode": "out"},
        "expected": {"distances": [0.0, 1.0, 3.0], "nrgeo": [1, 1, 1]},
    },
]

# ALGO-SP-005 A*: undirected triangle with shortcut.
ASTAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "a_star_path_py_undirected_triangle_shortcut",
        # Undirected triangle with weights (1,4,2): best 0→1→2 path.
        "origin": "constructed: triangle (1,4,2) with shortcut via vertex 1",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "a_star_path",
        "params": {"source": 0, "target": 2, "mode": "out"},
        "expected": {"vertices": [0, 1, 2], "edges": [0, 2]},
    },
]

# ALGO-SP-021..023 weighted: directed P3 OUT/IN/ALL.
ECC_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_weighted_py_directed_path_out",
        # Directed 0→1→2 weights (1, 2): OUT ecc = [3, 2, 0].
        "origin": "constructed: directed P3 weights (1, 2) — OUT mode",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "graph_weights": [1.0, 2.0],
        "algo": "eccentricity_weighted_with_mode",
        "params": {"mode": "out"},
        "expected": [3.0, 2.0, 0.0],
    },
]

RAD_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_weighted_py_undirected_triangle_shortcut",
        # Undirected triangle weights (1, 2, 4): vertex 1's ecc = 2 is min.
        "origin": "constructed: triangle weights (1,2,4), radius = 2.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 4.0],
        "algo": "radius_weighted_with_mode",
        "params": {"mode": "all"},
        "expected": 2.0,
    },
]

DIAM_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_weighted_py_directed_path_out",
        # Directed 0→1→2 weights (1, 2): OUT diameter = 3.
        "origin": "constructed: directed P3 (1, 2) — OUT diameter = 3.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "graph_weights": [1.0, 2.0],
        "algo": "diameter_weighted_with_mode",
        "params": {"mode": "out"},
        "expected": 3.0,
    },
]

MODULARITY_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_directed_py_3_cycle_single_partition",
        # python-igraph smoke style: directed 3-cycle with all
        # vertices in one community. e_norm = 1.0; k_out=k_in=1.0
        # → Q = 0.0.
        "origin": "constructed: directed 3-cycle, single community",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (2, 0)],
            directed=True,
        ),
        "algo": "modularity_directed",
        "params": {"membership": [0, 0, 0], "resolution": 1.0},
        "expected": 0.0,
    },
]

ASSORT_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_degree_directed_py_hub_in",
        # python-igraph smoke style: every vertex points at vertex 3
        # (a directed in-hub). Variance is well-defined on both sides.
        "origin": "constructed: directed hub 0/1/2/4 → 3",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (0, 2), (0, 3), (1, 3), (2, 3), (4, 3)],
            directed=True,
        ),
        "algo": "assortativity_degree_directed",
        "params": {},
        # Computed via python-igraph's
        # `g.assortativity_degree(directed=True)` → -0.7071067811865476.
        "expected": -0.7071067811865476,
    },
]

# ALGO-PR-006d: Directed weighted assortativity. Hand-computed
# reference value (no python API).
ASSORT_DIR_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_degree_directed_weighted_py_dag_diamond_unit",
        # DAG diamond 0→1, 0→2, 1→3, 2→3 with unit weights:
        # out_str=[2,1,1,0]; in_str=[0,1,1,2]. Hand-computed Pearson:
        # num1=8, num2=6, num3=6, den1=10, den2=10, total=4.
        # num = 8 - 36/4 = -1; var_from = var_to = 10 - 36/4 = 1.
        # r = -1 / sqrt(1*1) = -1.0.
        "origin": "constructed: DAG diamond unit weights, hand-computed r = -1.0",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (1, 3), (2, 3)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0, 1.0],
        "algo": "assortativity_degree_directed_weighted",
        "params": {},
        "expected": -1.0,
    },
]

CORENESS_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "coreness_with_mode_py_directed_3_cycle_in",
        # python-igraph smoke style: directed 3-cycle with mode='in'.
        # Each vertex has in-degree 1 → in-cores all 1.
        "origin": "constructed: directed 3-cycle 0→1→2→0, in-mode",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (2, 0)],
            directed=True,
        ),
        "algo": "coreness_with_mode",
        "params": {"mode": "in"},
        "expected": [1, 1, 1],
    },
]

DU_MANY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "disjoint_union_many_py_path_plus_path_plus_edge",
        # python-igraph smoke style: three graphs of varying sizes
        # disjoint-unioned. Output preserves vertex-shifts.
        "origin": "constructed: 2-path + 4-path + single edge",
        "graph_factory": lambda: ig.Graph(
            n=2,
            edges=[(0, 1)],
            directed=False,
        ),
        "algo": "disjoint_union_many",
        "params": {
            "extra_graphs": [
                {
                    "n": 4,
                    "edges": [[0, 1], [1, 2], [2, 3]],
                    "directed": False,
                    "weights": None,
                },
                {
                    "n": 2,
                    "edges": [[0, 1]],
                    "directed": False,
                    "weights": None,
                },
            ]
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "edges": [[0, 1], [2, 3], [3, 4], [4, 5], [6, 7]],
        },
    },
]

IS_SIMPLE_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_simple_with_mode_py_directed_3_cycle_undirected_view",
        # python-igraph smoke style: a directed 3-cycle 0→1→2→0
        # contains no mutual pairs, so the undirected view is also
        # simple (three distinct edges 0-1, 1-2, 0-2).
        "origin": "constructed: directed 3-cycle, undirected view → simple",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (2, 0)],
            directed=True,
        ),
        "algo": "is_simple_with_mode",
        "params": {"directed_as_undirected": True},
        "expected": True,
    },
]

MODULARITY_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_weighted_py_unit_weights",
        # python-igraph smoke style: unit weights collapse to
        # unweighted modularity, K3 ∪ K3 + bridge with [0,0,0,1,1,1]
        # → 6/7 - 0.5.
        "origin": "constructed: K3 ∪ K3 + bridge, unit weights",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
            directed=False,
        ),
        "graph_weights": [1.0] * 7,
        "algo": "modularity_weighted",
        "params": {"membership": [0, 0, 0, 1, 1, 1], "resolution": 1.0},
        "expected": 6.0 / 7.0 - 0.5,
    },
]

RECIP_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reciprocity_with_mode_py_ignore_loops_default",
        # python-igraph smoke style: directed with self-loop +
        # mutual pair. With ignore_loops=true, the self-loop drops
        # from both numerator and denominator → rec=2, denom=2 → 1.0.
        "origin": "constructed: self-loop 0→0 + mutual 0↔1, "
        "ignore_loops=true, default mode",
        "graph_factory": lambda: ig.Graph(
            n=2,
            edges=[(0, 0), (0, 1), (1, 0)],
            directed=True,
        ),
        "algo": "reciprocity_with_mode",
        "params": {"ignore_loops": True, "mode": "default"},
        "expected": 1.0,
    },
]

CORENESS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "coreness_py_k4_minus_edge",
        # python-igraph smoke style: K4 minus the (2, 3) edge. After
        # removing the missing edge, vertices 0 and 1 still have
        # degree 3 and form a denser sub-structure with each other and
        # with both 2, 3 — but 2 and 3 only have degree 2 each, so the
        # whole thing collapses to coreness 2 for everyone.
        "origin": "constructed: K4 minus edge (2,3)",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3)],
            directed=False,
        ),
        "algo": "coreness",
        "params": {},
        "expected": [2, 2, 2, 2],
    },
]

FW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "floyd_warshall_py_undirected_path_unit_weights",
        # python-igraph's `Graph.distances()` (no weights) returns the
        # full unweighted all-pairs matrix. On a 4-vertex path it is
        # the classic taxicab-on-a-line.
        "origin": "constructed: undirected 4-path 0-1-2-3, unit weights",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3)],
            directed=False,
        ),
        "algo": "floyd_warshall_distances",
        "params": {},
        "expected": [
            [0.0, 1.0, 2.0, 3.0],
            [1.0, 0.0, 1.0, 2.0],
            [2.0, 1.0, 0.0, 1.0],
            [3.0, 2.0, 1.0, 0.0],
        ],
    },
]

DISJOINT_UNION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "disjoint_union_py_path_plus_path",
        # python-igraph's `+` operator on graphs is disjoint_union.
        "origin": "constructed: 3-path + 2-path; vertex shift by 3",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "disjoint_union",
        "params": {
            "right_graph": {
                "n": 2,
                "edges": [[0, 1]],
                "directed": False,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "edges": [[0, 1], [1, 2], [3, 4]],
        },
    },
]

UNION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "union_py_triangle_overlap_path",
        # python-igraph: ig.union([K3, P4]) — vcount = max(3,4) = 4.
        # Triangle on {0,1,2} ∪ path 0-1-2-3 → max-multiplicity union
        # over the four canonical pairs {(0,1), (0,2), (1,2), (2,3)}.
        "origin": "constructed: K3 ∪ P4 on shared vertex space",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "algo": "union",
        "params": {
            "right_graph": {
                "n": 4,
                "edges": [[0, 1], [1, 2], [2, 3]],
                "directed": False,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 4,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 2], [2, 3]],
        },
    },
]

INTERSECTION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "intersection_py_triangle_overlap_path",
        # python-igraph: ig.intersection([K3, P4]) — vcount = max(3,4) = 4.
        # Triangle on {0,1,2} ∩ path 0-1-2-3 → only the shared pairs
        # (0,1) and (1,2) survive. (0,2) is in K3 but not in P4; (2,3)
        # is in P4 but not in K3.
        "origin": "constructed: K3 ∩ P4 on shared vertex space",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "algo": "intersection",
        "params": {
            "right_graph": {
                "n": 4,
                "edges": [[0, 1], [1, 2], [2, 3]],
                "directed": False,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 4,
            "directed": False,
            "edges": [[0, 1], [1, 2]],
        },
    },
]

DIFFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "difference_py_triangle_minus_path",
        # python-igraph: K3.difference(P4). orig = K3 on {0,1,2}, sub =
        # path 0-1-2-3 on 4 vertices. Per canonicalised undirected pair:
        #   (0,1): 1−1=0, (1,2): 1−1=0, (0,2): 1−0=1; (2,3) is in sub
        #   only and is ignored. vcount = orig.vcount() = 3 (asymmetric).
        "origin": "constructed: K3 \\ P4 on shared vertex space",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "algo": "difference",
        "params": {
            "right_graph": {
                "n": 4,
                "edges": [[0, 1], [1, 2], [2, 3]],
                "directed": False,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 3,
            "directed": False,
            "edges": [[0, 2]],
        },
    },
]

IS_LOOP_PER_EDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_loop_py_no_self_loops",
        "origin": "constructed: 3-edge path; per-edge is_loop all False",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "is_loop",
        "params": {},
        "expected": [False, False, False],
    },
]

IS_MULTIPLE_PER_EDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_multiple_py_simple_path",
        "origin": "constructed: 3-edge simple path; is_multiple all False",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "is_multiple",
        "params": {},
        "expected": [False, False, False],
    },
]

HAS_LOOP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "has_loop_py_simple_path_no_loop",
        "origin": "constructed: 4-path no loops; has_loop=false",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "has_loop",
        "params": {},
        "expected": False,
    },
]

HAS_MULTIPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "has_multiple_py_simple_path_no_multi",
        "origin": "constructed: 4-path no multi-edges; has_multiple=false",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "has_multiple",
        "params": {},
        "expected": False,
    },
]

COUNT_LOOPS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_loops_py_simple_path",
        # python-igraph: Graph.has_multiple / Graph.is_loop have idiomatic
        # counterparts; count_loops here is sum(g.is_loop()).
        "origin": "constructed: 4-path no self-loops; count_loops=0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "count_loops",
        "params": {},
        "expected": 0,
    },
    {
        "case": "count_loops_py_two_self_loops_directed",
        "origin": "constructed: directed (0,0)(1,1)(0,1); count_loops=2",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 0), (1, 1), (0, 1)], directed=True
        ),
        "algo": "count_loops",
        "params": {},
        "expected": 2,
    },
]

COUNT_MULTIPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_multiple_py_simple_path_all_ones",
        # Plain undirected path → every edge is alone in its pair group.
        "origin": "constructed: undirected path 0-1-2-3; multiplicity = [1,1,1]",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "count_multiple",
        "params": {},
        "expected": [1, 1, 1],
    },
    {
        "case": "count_multiple_py_three_parallel_undirected",
        "origin": "constructed: three parallel undirected (0,1); multiplicity = [3,3,3]",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1), (0, 1)], directed=False
        ),
        "algo": "count_multiple",
        "params": {},
        "expected": [3, 3, 3],
    },
]

COUNT_ADJACENT_TRIANGLES_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_adjacent_triangles_py_triangle",
        # python-igraph: g.count_adjacent_triangles() returns per-vertex
        # adjacent-triangle counts. For a triangle every vertex sees one.
        "origin": "constructed: undirected triangle (0,1)(1,2)(2,0); per-vertex = [1,1,1]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "count_adjacent_triangles",
        "params": {},
        "expected": [1, 1, 1],
    },
    {
        "case": "count_adjacent_triangles_py_star_zero",
        # Star K_{1,4}: no triangles at all.
        "origin": "constructed: undirected star, centre 0 with 4 leaves; per-vertex = [0,0,0,0,0]",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (0, 2), (0, 3), (0, 4)], directed=False
        ),
        "algo": "count_adjacent_triangles",
        "params": {},
        "expected": [0, 0, 0, 0, 0],
    },
]

IS_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_simple_py_self_loop_not_simple",
        # Self-loop disqualifies → not simple. Mirrors the trivial case
        # asserted throughout python-igraph's structural test suite.
        "origin": "constructed: graph with one self-loop; not simple",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 0), (1, 2)], directed=False
        ),
        "algo": "is_simple",
        "params": {},
        "expected": False,
    },
]

# Louvain (ALGO-CO-002). python-igraph exposes Louvain as
# `Graph.community_multilevel`. Same gain formula as upstream C — its
# partition varies with shuffle order, so the conformance harness
# asserts on a modularity range and a community-count window, not on
# exact membership.
LOUVAIN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "louvain_py_karate",
        # The canonical python-igraph test: karate club from
        # ig.Graph.Famous("Zachary"). community_multilevel hits
        # Q ≈ 0.39..0.42; partition consistently lands on 4 communities.
        "origin": "tests/test_clustering.py CommunityDetectionTests "
        "(Famous('Zachary'), community_multilevel); Q ≈ 0.39..0.42, k=4",
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
        "case": "louvain_py_full5_full5_bridge",
        # K5 + K5 + bridge — the same fixture used by test_structural's
        # modularity test, but here we let Louvain *find* the partition.
        # Two K5s dominated by 10 internal edges each vs 1 bridge edge;
        # Louvain must split, k=2.
        "origin": "test_structural.py-style K5+K5+bridge run through "
        "community_multilevel; k=2, Q ≈ 0.45",
        "graph_factory": lambda: ig.Graph.Full(5) + ig.Graph.Full(5) + [(0, 5)],
        "algo": "louvain",
        "params": {"resolution": 1.0},
        "expected": {
            "modularity_min": 0.42,
            "modularity_max": 0.47,
            "k_min": 2,
            "k_max": 3,
        },
    },
]

# Leiden (ALGO-CO-003). python-igraph entrypoint:
# `Graph.community_leiden`. Same Q-range + k-window oracle as Louvain;
# Leiden is non-deterministic across implementations so we never assert
# exact membership.
LEIDEN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "leiden_py_karate",
        "origin": "tests/test_clustering.py CommunityDetectionTests "
        "(Famous('Zachary'), community_leiden); modularity objective, "
        "Q ≈ 0.39..0.45, k ≈ 4",
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
        "case": "leiden_py_full5_full5_bridge",
        "origin": "test_structural.py-style K5+K5+bridge run through "
        "community_leiden; k=2, Q ≈ 0.45",
        "graph_factory": lambda: ig.Graph.Full(5) + ig.Graph.Full(5) + [(0, 5)],
        "algo": "leiden",
        "params": {"objective": "modularity", "resolution": 1.0},
        "expected": {
            "modularity_min": 0.42,
            "modularity_max": 0.47,
            "k_min": 2,
            "k_max": 3,
        },
    },
]

WALKTRAP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "walktrap_py_karate",
        # python-igraph: g.community_walktrap(steps=4).as_clustering().
        # On Famous('Zachary') the Pons-Latapy walk picks Q ∈ [0.30, 0.45]
        # with k ∈ [3, 6] (tie-break varies a tick across ports).
        "origin": "Famous('Zachary'); community_walktrap steps=4; "
        "Q ∈ [0.30, 0.45], k ∈ [3, 6]",
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
        "case": "walktrap_py_k5_k5_bridge",
        # K5+K5 joined by a bridge: Walktrap recovers the two cliques at
        # k = 2 with Q ≈ 0.45 (same envelope as the fast-greedy mirror).
        "origin": "K5+K5+bridge (0,5); community_walktrap steps=4; "
        "k=2, Q ≈ 0.42..0.47",
        "graph_factory": lambda: ig.Graph.Full(5) + ig.Graph.Full(5) + [(0, 5)],
        "algo": "walktrap",
        "params": {"steps": 4},
        "expected": {
            "modularity_min": 0.42,
            "modularity_max": 0.47,
            "k_min": 2,
            "k_max": 2,
        },
    },
    {
        "case": "walktrap_py_ring6_weighted",
        # Mirror of the C reference ring-6 weighted output: weights
        # [1.0, 0.5, 0.25, 0.75, 1.25, 1.5] on a 6-cycle. Walktrap with
        # steps=4 best-cuts at Q ≈ 0.146 with k = 3.
        "origin": "constructed: 6-ring + weights [1,0.5,0.25,0.75,1.25,1.5]; "
        "community_walktrap.out best Q = 0.146259, k = 3",
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
        "case": "fastgreedy_py_karate",
        # python-igraph: g.community_fastgreedy().as_clustering().
        # On Famous('Zachary') python-igraph reports Q ≈ 0.38 with
        # k ∈ [2, 5] across versions.
        "origin": "Famous('Zachary'); community_fastgreedy; "
        "Q ∈ [0.30, 0.45], k ∈ [2, 5]",
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
        "case": "fastgreedy_py_k5_k5_bridge",
        # K5+K5+bridge: fast-greedy cleanly recovers the two K5s,
        # Q ≈ 0.452 (≈ the C unit-test value).
        "origin": "K5+K5+bridge (0,5); community_fastgreedy "
        "k=2; Q ≈ 0.42..0.47",
        "graph_factory": lambda: ig.Graph.Full(5) + ig.Graph.Full(5) + [(0, 5)],
        "algo": "fast_greedy_modularity",
        "params": {},
        "expected": {
            "modularity_min": 0.42,
            "modularity_max": 0.47,
            "k_min": 2,
            "k_max": 2,
        },
    },
]

EB_COMMUNITY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eb_community_py_karate",
        # python-igraph: g.community_edge_betweenness() — Newman-Girvan.
        # On Famous('Zachary') the partition lands at Q ≈ 0.40, k ∈ [2, 5].
        "origin": "Famous('Zachary'); community_edge_betweenness; "
        "Q ∈ [0.30, 0.45], k ∈ [2, 5]",
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
        "case": "eb_community_py_k5_k5_bridge",
        # K5+K5+bridge: the bridge is the highest-betweenness edge; the
        # algorithm removes it first and yields a Q ≈ 0.45 partition.
        "origin": "K5+K5+bridge (0,5); community_edge_betweenness "
        "k=2; Q ≈ 0.40..0.47",
        "graph_factory": lambda: ig.Graph.Full(5) + ig.Graph.Full(5) + [(0, 5)],
        "algo": "edge_betweenness_community",
        "params": {},
        "expected": {
            "modularity_min": 0.40,
            "modularity_max": 0.47,
            "k_min": 2,
            "k_max": 2,
        },
    },
    {
        "case": "eb_community_py_directed_path_6",
        # Directed 6-path 0→1→2→3→4→5: edge (2,3) carries the unique
        # max directed betweenness ⇒ first removal ⇒ {0,1,2}|{3,4,5}.
        # Directed Q = 8/25 = 0.32 by hand.
        "origin": "directed 6-path; community_edge_betweenness; "
        "k=2; directed Q = 8/25 ≈ 0.32",
        "graph_factory": lambda: ig.Graph(
            n=6, edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)], directed=True
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

# Weighted edge-betweenness community (ALGO-CO-006b). python-igraph maps
# to `g.community_edge_betweenness(weights=...)`. Unit-weight invocations
# must reproduce the unweighted dendrogram identically; non-unit weights
# bias the per-removal Brandes pass toward cheap-bridge first removals.
EB_COMMUNITY_WEIGHTED_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eb_community_weighted_py_karate_unit",
        "origin": "Famous('Zachary'); community_edge_betweenness(weights=[1]*78); "
        "Q ∈ [0.30, 0.45], k ∈ [2, 5]",
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
        "case": "eb_community_weighted_py_k5_k5_bridge_unit",
        # Same K5+K5+bridge as the unweighted slice but routed through the
        # weighted pipeline with unit weights ⇒ identical Q envelope.
        "origin": "K5+K5+bridge (0,5); community_edge_betweenness(weights=[1]*21); "
        "k=2; Q ∈ [0.40, 0.47]",
        "graph_factory": lambda: ig.Graph.Full(5) + ig.Graph.Full(5) + [(0, 5)],
        "graph_weights": [1.0] * 21,
        "algo": "edge_betweenness_community_weighted",
        "params": {},
        "expected": {
            "modularity_min": 0.40,
            "modularity_max": 0.47,
            "k_min": 2,
            "k_max": 2,
        },
    },
    {
        "case": "eb_community_weighted_py_directed_path_6_unit",
        # Directed 6-path with unit weights ⇒ same dendrogram as the
        # unweighted directed slice. Directed-weighted Q = 8/25 ≈ 0.32.
        "origin": "directed 6-path; community_edge_betweenness(weights=[1]*5); "
        "k=2; directed-weighted Q = 8/25 ≈ 0.32",
        "graph_factory": lambda: ig.Graph(
            n=6, edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)], directed=True
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
        "case": "fluid_py_karate_k2",
        "origin": "Famous('Zachary'); community_fluid_communities(k=2); "
        "Q ∈ [0.20, 0.42]",
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
        "case": "fluid_py_k5_k5_bridge_k2",
        # K5+K5 joined by a single edge: Fluid with k=2 cleanly cuts the
        # bridge; Q ≈ 0.4523 by hand.
        "origin": "K5+K5+bridge run through community_fluid_communities(k=2); "
        "Q ≈ 0.40..0.47",
        "graph_factory": lambda: ig.Graph.Full(5) + ig.Graph.Full(5) + [(0, 5)],
        "algo": "fluid_communities",
        "params": {"k": 2},
        "expected": {
            "modularity_min": 0.40,
            "modularity_max": 0.47,
            "k_min": 2,
            "k_max": 2,
        },
    },
]

LPA_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lpa_py_karate",
        "origin": "tests/test_clustering.py CommunityDetectionTests "
        "(Famous('Zachary'), community_label_propagation); LPA Q ∈ [0.20, 0.42], "
        "k ∈ [2, 10] (variant-independent envelope)",
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
        "case": "lpa_py_full5_full5_bridge",
        # K5+K5 joined by a single edge: LPA reliably yields the
        # natural 2-community split; Q ≈ 0.4523 from ground truth.
        "origin": "test_structural-style K5+K5+bridge run through "
        "community_label_propagation; k=2, Q ≈ 0.40..0.47",
        "graph_factory": lambda: ig.Graph.Full(5) + ig.Graph.Full(5) + [(0, 5)],
        "algo": "label_propagation",
        "params": {},
        "expected": {
            "modularity_min": 0.40,
            "modularity_max": 0.47,
            "k_min": 2,
            "k_max": 3,
        },
    },
]

MODULARITY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_py_full5_full5_bridge",
        # From python-igraph tests/test_structural.py:127 testModularity:
        # Graph.Full(5) + Graph.Full(5) + edge (0,5); membership
        # [0]*5 + [1]*5; expected ≈ 0.4523.
        "origin": "test_structural.py:127 testModularity: K5 + K5 + bridge; "
        "Q ≈ 0.4523",
        "graph_factory": lambda: ig.Graph.Full(5)
        + ig.Graph.Full(5)
        + [(0, 5)],
        "algo": "modularity",
        "params": {
            "membership": [0] * 5 + [1] * 5,
            "resolution": 1.0,
        },
        # Computed via python-igraph 0.11 to f64 precision.
        "expected": 0.45238095238095233,
    },
]

SIMPLIFY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "simplify_py_loops_no_multi",
        # From python-igraph tests/test_operators.py: simplify(loops=False)
        # only removes parallel edges; loops survive. We seed (0,0)(0,1)(0,1)
        # → simplify(multiple=True, loops=False) keeps the loop and one (0,1).
        "origin": "test_operators.py:432: g.simplify(loops=False) only removes multi-edges",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 0), (0, 1), (0, 1)], directed=False
        ),
        "algo": "simplify",
        "params": {"remove_multiple": True, "remove_loops": False},
        "expected": {"vcount": 2, "directed": False, "edges": [[0, 0], [0, 1]]},
    },
]

TC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitive_closure_py_undirected_path2",
        # 2-vertex undirected single edge: closure equals input.
        "origin": "constructed: undirected 2-path; closure = input",
        "graph_factory": lambda: ig.Graph(n=2, edges=[(0, 1)], directed=False),
        "algo": "transitive_closure",
        "params": {},
        "expected": {
            "vcount": 2,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
]

REACH_MATRIX_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reachability_matrix_py_undirected_path3",
        # 3-vertex undirected path: all pairs reachable (within one component).
        "origin": "constructed: undirected path 0-1-2; full True matrix (1 component)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "reachability_matrix",
        "params": {},
        "expected": [[True, True, True], [True, True, True], [True, True, True]],
    },
]

REACH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_reachable_two_components",
        # Undirected two components {0,1,2} and {3,4}: counts [3,3,3,2,2].
        "origin": "constructed: two components — counts = component sizes",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (3, 4)], directed=False
        ),
        "algo": "count_reachable",
        "params": {},
        "expected": [3, 3, 3, 2, 2],
    },
]

ASSORT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_diamond_K4_minus_edge",
        # K4 - edge(0,3): expected -2/3 (verified above; matches python-igraph
        # exactly).
        "origin": "constructed: K4 - edge(0,3); assortativity_degree = -2/3",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "algo": "assortativity_degree",
        "params": {},
        "expected": -0.6666666666666728,
    },
]

DENSITY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "density_K4",
        # K4: 6 edges among 6 possible → 1.0.
        "origin": "constructed: K4; density 1.0",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "density",
        "params": {},
        "expected": 1.0,
    },
]

MEANDIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mean_distance_path_5",
        # Path 0-1-2-3-4: mean = 2.0 (computed above).
        "origin": "constructed: 5-path; mean distance 2.0",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "mean_distance",
        "params": {},
        "expected": 2.0,
    },
]

GLOBAL_EFFICIENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "global_efficiency_py_star_4",
        # Star K_{1,3}: centre at d=1 from leaves; leaves at d=2 to each
        # other. 12 ordered pairs: 6 at d=1 (between centre/leaf both ways),
        # 6 at d=2 (between leaves both ways). Sum 1/d = 6 + 3 = 9; /12.
        "origin": "constructed: star K_{1,3}; global_efficiency=0.75",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "global_efficiency",
        "params": {},
        "expected": 0.75,
    },
    {
        "case": "global_efficiency_py_disconnected",
        # {0-1}, {2}: 2 reachable pairs at d=1; 4 unreachable. Sum=2; /6.
        "origin": "constructed: edge (0,1) plus isolated 2; global_efficiency=1/3",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1)], directed=False
        ),
        "algo": "global_efficiency",
        "params": {},
        "expected": 1.0 / 3.0,
    },
]

LTRANS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_local_star",
        # Star: centre has clustering 0; leaves have None (degree<2).
        "origin": "constructed: 4-vertex star — centre 0.0, leaves None (deg<2)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "transitivity_local_undirected",
        "params": {},
        "expected": [0.0, None, None, None],
    },
]

TRANS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_diamond_K4_minus_edge",
        # K4 minus edge (0,3): triangles=2, triples=8, transitivity=6/8=0.75.
        # Verified directly via python-igraph 0.11.
        "origin": "constructed: K4 minus edge (0,3); transitivity = 0.75",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "algo": "transitivity_undirected",
        "params": {},
        "expected": 0.75,
    },
]

DIAM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_4_cycle",
        # 4-cycle has diameter 2 (longest geodesic between opposite vertices).
        "origin": "constructed: 4-cycle via python-igraph; diameter 2",
        "graph_factory": lambda: ig.Graph.Ring(
            n=4, directed=False, mutual=False, circular=True
        ),
        "algo": "diameter",
        "params": {},
        "expected": 2,
    },
]

ECC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_star_4",
        # Star with centre 0 and 3 leaves: ecc = [1, 2, 2, 2].
        "origin": "constructed: 4-vertex star via python-igraph; eccentricity vector",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "eccentricity",
        "params": {},
        "expected": [1, 2, 2, 2],
    },
]

RAD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_star_4",
        "origin": "constructed: 4-vertex star — radius 1 (centre)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "radius",
        "params": {},
        "expected": 1,
    },
]

ECC_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_with_mode_py_directed_cycle3_all",
        # Directed 3-cycle 0→1→2→0 under "all" mode → underlying graph
        # is K3 (triangle), so every vertex has eccentricity 1.
        "origin": "constructed: directed 3-cycle — ALL-mode collapses to undirected K3",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "eccentricity_with_mode",
        "params": {"mode": "all"},
        "expected": [1, 1, 1],
    },
]

RAD_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_with_mode_py_directed_cycle3_all",
        "origin": "constructed: directed 3-cycle — ALL-mode radius = 1 (uniform K3)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "radius_with_mode",
        "params": {"mode": "all"},
        "expected": 1,
    },
]

DIAM_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_with_mode_py_directed_cycle3_all",
        "origin": "constructed: directed 3-cycle — ALL-mode diameter = 1 (uniform K3)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "diameter_with_mode",
        "params": {"mode": "all"},
        "expected": 1,
    },
]

GIRTH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "girth_petersen_like_pentagon",
        # python-igraph's Famous("Petersen") is too big for a fixture;
        # use a 5-cycle which trivially has girth 5.
        "origin": "constructed: 5-cycle via python-igraph; girth = 5",
        "graph_factory": lambda: ig.Graph.Ring(
            n=5, directed=False, mutual=False, circular=True
        ),
        "algo": "girth",
        "params": {},
        "expected": 5,
    },
]

ISBI_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_biconnected_k4",
        # K4 (complete graph on 4 vertices): biconnected.
        "origin": "constructed: K4 via python-igraph; biconnected per Graph.is_biconnected()",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "is_biconnected",
        "params": {},
        "expected": True,
    },
]

BRIDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bridges_4_path_all_edges",
        # No verbatim test in test_decomposition.py. Constructed: a 4-vertex
        # path 0-1-2-3 has all 3 edges as bridges. Verified via
        # python-igraph 0.11 g.bridges() returning [0, 1, 2].
        "origin": "constructed: 4-vertex path; expected via python-igraph 0.11 Graph.bridges()",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "bridges",
        "params": {},
        "expected": [0, 1, 2],
    },
]

AP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "tree_5_2_articulation",
        # python-igraph's Tree(5,2) yields edges (0,1),(0,2),(1,3),(1,4) →
        # AP = {0, 1}. Verified directly against python-igraph 0.11.
        "origin": "constructed: python-igraph Tree(5,2); AP via Graph.articulation_points()",
        "graph_factory": lambda: _tree(5, 2),
        "algo": "articulation_points",
        "params": {},
        "expected": [0, 1],
    },
]

DIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "tree10_2_source0",
        # python-igraph's test_iterators.testBFS uses Tree(10, 2) as the
        # canonical small BFS fixture. We re-use it for distances; the
        # expected vector is just the BFS layer index of each vertex.
        "origin": "test_iterators.py:IteratorTests.testBFS Tree(10,2) — "
        "distances(source=0) layer indices, verified against python-igraph 0.11",
        "graph_factory": lambda: _tree(10, 2),
        "algo": "distances",
        "params": {"source": 0},
        "expected": [0, 1, 1, 2, 2, 2, 2, 3, 3, 3],
    },
]

SCC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "directed_4cycle_with_tail",
        # No verbatim test in test_decomposition.py; constructed to
        # exercise the post-order pass-2 distinction (cycle vs dangling
        # vertex). Expected pulled from python-igraph 0.11 directly.
        "origin": "constructed: directed 4-cycle 0->1->2->3->0 plus 3->4 tail; "
        "expected verified against python-igraph 0.11 connected_components(mode='strong')",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (1, 2), (2, 3), (3, 0), (3, 4)],
            directed=True,
        ),
        "algo": "strongly_connected_components",
        "params": {},
        "expected": {"membership": [0, 0, 0, 0, 1], "count": 2},
    },
]

COMMUNITY_TO_MEMBERSHIP_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph `VertexDendrogram.as_clustering(n=...)` is the
    # documented user-facing wrapper around the C
    # `igraph_community_to_membership`. It cuts at a specified cluster
    # count, internally invoking the same top-down densify pass. The
    # fixtures below are computed via that wrapper on small
    # hand-constructed dendrograms; cluster labels may differ from the
    # Rust impl (the conformance test compares partitions canonically).
    {
        "case": "community_to_membership_py_one_merge_untouched",
        "origin": "python-igraph VertexDendrogram.as_clustering: 5 leaves, "
        "one merge [[0,2]], steps=1 -> {0,2} cluster + singletons {1},{3},{4}",
        "nodes": 5,
        "merges": [[0, 2]],
        "steps": 1,
        "expected": {"membership": [0, 1, 0, 2, 3], "csize": [2, 1, 1, 1]},
    },
    {
        "case": "community_to_membership_py_six_three_parallel",
        "origin": "python-igraph VertexDendrogram.as_clustering: 6 leaves, "
        "three parallel merges [[0,1],[2,3],[4,5]], steps=3 -> three pairs",
        "nodes": 6,
        "merges": [[0, 1], [2, 3], [4, 5]],
        "steps": 3,
        "expected": {"membership": [0, 0, 1, 1, 2, 2], "csize": [2, 2, 2]},
    },
]

COMPARE_COMMUNITIES_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph exposes `compare_communities(comm1, comm2, method)`
    # at module level (and as the `compare_to` method of
    # `VertexClustering`). Values below are computed via
    # `igraph.compare_communities(...)` and reproduced here as fixed
    # references; the conformance test asserts the Rust result is
    # within 1e-9 of these values.
    {
        "case": "compare_communities_py_split_join_subpartition",
        "origin": "python-igraph igraph.compare_communities(method='split_join'): "
        "refinement (b refines a) — d12=2, d21=0, sum=2.",
        "comm1": [0, 0, 0, 1, 1, 1],
        "comm2": [5, 5, 6, 7, 7, 8],
        "method": "split_join",
        "expected": {"value": 2.0},
    },
    {
        "case": "compare_communities_py_adjusted_rand_full_disagreement",
        "origin": "python-igraph igraph.compare_communities(method='adjusted_rand'): "
        "2x2 full-disagreement confusion (n=4) — AR = -0.5.",
        "comm1": [0, 0, 1, 1],
        "comm2": [0, 1, 0, 1],
        "method": "adjusted_rand",
        "expected": {"value": -0.5},
    },
]

SPLIT_JOIN_DISTANCE_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph does not expose the asymmetric pair directly
    # (`igraph.compare_communities(method='split_join')` returns the
    # symmetric scalar `d12 + d21`). The pair fixtures below are fixed
    # references derived from the upstream confusion-matrix
    # decomposition; the sum `d12 + d21` matches
    # `igraph.compare_communities(method='split_join')` on the same
    # inputs.
    {
        "case": "split_join_distance_py_refinement",
        "origin": "python-igraph igraph.compare_communities(method='split_join') parity: "
        "comm2 strictly refines comm1 (b splits each a-cluster into 2-1) ⇒ d12=2, d21=0, sum=2.",
        "comm1": [0, 0, 0, 1, 1, 1],
        "comm2": [5, 5, 6, 7, 7, 8],
        "expected": {"d12": 2, "d21": 0},
    },
    {
        "case": "split_join_distance_py_full_disagreement_2x2",
        "origin": "python-igraph igraph.compare_communities(method='split_join') parity: "
        "2x2 full-disagreement (n=4) — d12=d21=2, sum=4.",
        "comm1": [0, 0, 1, 1],
        "comm2": [0, 1, 0, 1],
        "expected": {"d12": 2, "d21": 2},
    },
]

VORONOI_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph 0.11 does not expose `igraph_voronoi` as a bound
    # method (the C function exists, but there is no Python wrapper as
    # of this writing). The fixtures below are derived at extraction
    # time using `Graph.distances()` plus an in-script FIRST / LAST
    # tiebreaker pass — that way the "Python source" is still
    # python-igraph (the BFS / Dijkstra inner loop), just stitched
    # together with the published Voronoi-cell tiebreaker rule from
    # `references/igraph/src/paths/voronoi.c` lines 30-309.
    #
    # The expected `membership` / `distances` fields are filled in by
    # the dedicated `voronoi` branch in `emit()` below.
    {
        "case": "voronoi_py_path5_endpoints_first",
        "origin": "python-igraph Graph.distances() + FIRST tiebreaker: "
        "undirected path P5 with generators=[0,4] — vertex 2 ties (dist=2 to both); FIRST keeps 0.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "params": {
            "generators": [0, 4],
            "mode": "all",
            "tiebreaker": "first",
        },
    },
    {
        "case": "voronoi_py_path5_endpoints_last",
        "origin": "python-igraph Graph.distances() + LAST tiebreaker: "
        "undirected path P5 with generators=[0,4] — vertex 2 ties (dist=2); LAST flips it to 4.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "params": {
            "generators": [0, 4],
            "mode": "all",
            "tiebreaker": "last",
        },
    },
    {
        "case": "voronoi_py_karate_first",
        "origin": "python-igraph Graph.distances() + FIRST tiebreaker on Zachary karate club: "
        "generators=[0,32,24], mode=ALL, unweighted.",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "params": {
            "generators": [0, 32, 24],
            "mode": "all",
            "tiebreaker": "first",
        },
    },
]

ECC_PR031_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph 0.11 does not expose `igraph_ecc`. The fixtures
    # below are hand-derived from Radicchi 2004's definition,
    # `C^(k)_ij = (z + offset) / s` with `s_3 = min(d_i,d_j) - 1` and
    # `s_4 = (d_i-1)(d_j-1)`. They are small enough that the expected
    # values follow directly from manual neighbour enumeration, so the
    # parity check is genuinely meaningful even without a python-igraph
    # binding to call.
    {
        "case": "ecc_py_k3_triangle_normalized",
        # K_3: every edge sits in 1 triangle (z=1), every vertex has
        # degree 2 → s = 1. C = 1/1 = 1 for all 3 edges.
        "origin": "constructed: K_3, k=3, offset=false, normalize=true → all 1.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "ecc",
        "params": {"k": 3, "offset": False, "normalize": True},
        "expected": [1.0, 1.0, 1.0],
    },
    {
        "case": "ecc_py_k3_k4_normalized_offset_false",
        # K_4: every edge sits in 2 triangles (z=2), degrees all 3 →
        # s = 2. C = 1 for all 6 edges.
        "origin": "constructed: K_4, k=3, offset=false, normalize=true → all 1.0",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "ecc",
        "params": {"k": 3, "offset": False, "normalize": True},
        "expected": [1.0] * 6,
    },
    {
        "case": "ecc_py_k4_k4_offset_false_normalize_true",
        # K_4 at k=4: each non-edge endpoint contributes one 4-cycle,
        # giving z=2, s=(3-1)*(3-1)=4 → 0.5.
        "origin": "constructed: K_4, k=4, offset=false, normalize=true → all 0.5",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "ecc",
        "params": {"k": 4, "offset": False, "normalize": True},
        "expected": [0.5] * 6,
    },
    {
        "case": "ecc_py_p2_offset_false_normalize_true_is_nan",
        # P_2 (single edge): s = min(1,1) - 1 = 0 → NaN.
        "origin": "constructed: P_2, k=3, normalize=true → NaN (s = 0)",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1)], directed=False
        ),
        "algo": "ecc",
        "params": {"k": 3, "offset": False, "normalize": True},
        "expected": [None],
    },
]


# python-igraph 0.11 does not expose `igraph_rich_club_sequence`. The
# fixtures below are hand-derived from the algorithm's documented
# definition (Zhou & Mondragón 2004): each output position i is the
# rich-club coefficient of the subgraph induced by
# `vertex_order[i:]`, equal to the count of remaining edges (or sum of
# remaining weights) divided by `total_possible_edges(k, directed,
# loops)`, where `k = n - i`. NaN is encoded as JSON `null` (the
# runner converts NaN ↔ null both ways).
RICH_CLUB_MANIFEST: List[Dict[str, Any]] = [
    {
        # Triangle K_3, in-order removal; normalized; loops=false;
        # directed=false. Subgraphs are K_3, K_2, K_1: 3/3, 1/1, 0/0.
        "case": "rich_club_py_k3_inorder_normalized",
        "origin": "constructed: K_3, vertex_order=[0,1,2], normalized → [1.0, 1.0, NaN]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [0, 1, 2],
            "normalized": True,
            "loops": False,
            "directed": False,
        },
        "expected": [1.0, 1.0, None],
    },
    {
        # 4-path 0—1—2—3, in-order removal; normalized; loops=false;
        # directed=false. Edges left after removing prefix:
        #   i=0: 3 edges, max 4*3/2 = 6 → 0.5
        #   i=1: 2 edges, max 3*2/2 = 3 → 2/3
        #   i=2: 1 edge,  max 2*1/2 = 1 → 1
        #   i=3: 0 edges, max 0       → NaN
        "case": "rich_club_py_path4_inorder_normalized",
        "origin": "constructed: P_4 0-1-2-3, vertex_order=[0..3], normalized",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [0, 1, 2, 3],
            "normalized": True,
            "loops": False,
            "directed": False,
        },
        "expected": [3 / 6, 2 / 3, 1.0, None],
    },
    {
        # Weighted star K_{1,3} centred on vertex 0, weights [1, 2, 4].
        # vertex_order=[1, 2, 3, 0] removes leaves first so the centre
        # lives until the end.
        #   i=0: full graph, weight = 1+2+4 = 7, max = 4*3/2 = 6 → 7/6
        #   i=1: subgraph on {2,3,0}, edges (0,2)w=2 (0,3)w=4 → 6, max 3*2/2=3 → 2.0
        #   i=2: subgraph on {3,0},   edge  (0,3)w=4         → 4, max 2*1/2=1 → 4.0
        #   i=3: subgraph on {0} only → 0, max 0 → NaN
        "case": "rich_club_py_star_weighted_centre_last",
        "origin": "constructed: weighted K_{1,3}, peel leaves first → centre last",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 4.0],
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [1, 2, 3, 0],
            "normalized": True,
            "loops": False,
            "directed": False,
        },
        "expected": [7 / 6, 2.0, 4.0, None],
    },
]


COMMUNITY_VORONOI_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph 0.11 does not expose `igraph_community_voronoi`.
    # The fixtures below are hand-derived from the algorithm's
    # documented contract (Deritei 2014 / Molnár 2024) and cross-checked
    # against the C reference `.out`. All assertions are on `generators`
    # + community count, not raw membership labels — the latter depend
    # on the random tiebreaker inside the inner `voronoi` call, which is
    # not reproducible across distinct PRNG families.
    {
        "case": "community_voronoi_py_null",
        "origin": "constructed: null graph (n=0) — empty output",
        "graph_factory": lambda: ig.Graph(n=0, edges=[], directed=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": -1.0},
        "expected": {"generators": [], "community_count": 0},
    },
    {
        "case": "community_voronoi_py_singleton",
        "origin": "constructed: single isolated vertex — self-generator",
        "graph_factory": lambda: ig.Graph(n=1, edges=[], directed=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": -1.0},
        "expected": {"generators": [0], "community_count": 1},
    },
    {
        "case": "community_voronoi_py_k4_fixed_r0",
        # K_4 with r=0: every other vertex is at strictly positive
        # distance from any generator, so each pick excludes only the
        # generator itself → 4 generators, 4 singleton communities.
        # LRD is uniform across K_4 so ties break by vertex id (0,1,2,3).
        "origin": "constructed: K_4 with fixed r=0 — 4 singleton communities",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": 0.0},
        "expected": {"generators": [0, 1, 2, 3], "community_count": 4},
    },
    {
        "case": "community_voronoi_py_k5_fixed_r0",
        # Same as above for K_5 — independent verification at a
        # different size.
        "origin": "constructed: K_5 with fixed r=0 — 5 singleton communities",
        "graph_factory": lambda: ig.Graph.Full(n=5, directed=False, loops=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": 0.0},
        "expected": {"generators": [0, 1, 2, 3, 4], "community_count": 5},
    },
]


REINDEX_MEMBERSHIP_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph does not expose `igraph_reindex_membership`
    # directly; the closest user-facing analogue is
    # `VertexClustering(... reindex=True)` (and `Graph.clusters()` /
    # `community_*` results, all of which return a densified
    # membership). The fixtures below encode the same first-occurrence
    # semantics — they are partition-equivalent to what the C / R
    # implementations produce, and the conformance test uses
    # canonical relabelling, so cluster id ordering differences are
    # absorbed.
    {
        "case": "reindex_membership_py_first_occurrence",
        "origin": "python-igraph VertexClustering(reindex=True) parity: "
        "out-of-order ids relabelled in first-encounter order",
        "membership": [2, 2, 0, 1, 0, 2, 1],
        "expected": {"membership": [0, 0, 1, 2, 1, 0, 2], "new_to_old": [2, 0, 1]},
    },
    {
        "case": "reindex_membership_py_singletons",
        "origin": "python-igraph VertexClustering(reindex=True) parity: "
        "every vertex in its own cluster — each gets a fresh id",
        "membership": [10, 11, 12, 13, 14],
        "expected": {
            "membership": [0, 1, 2, 3, 4],
            "new_to_old": [10, 11, 12, 13, 14],
        },
    },
]

# ALGO-MST-001: minimum_spanning_tree. python-igraph exposes
# `Graph.spanning_tree(weights=None, return_tree=False)` returning a list
# of edge IDs. We compare on the matroid invariant — total weight + edge
# count — rather than exact edge IDs to absorb tie-break differences
# between Kruskal/Prim/Unweighted/Automatic. The `method` param picks
# the Rust dispatch path.
SPANNING_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "spanning_tree_py_k4_distinct_weights_kruskal",
        # K4 with all six distinct edge weights → unique MST (edges
        # 0,1,2 of weight 1+2+3=6).
        "origin": "constructed: K4 with edge weights [1..6], "
        "kruskal picks the three lightest",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "graph_weights": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        "algo": "minimum_spanning_tree",
        "params": {"method": "kruskal"},
        "expected": {"total_weight": 6.0, "edge_count": 3},
    },
    {
        "case": "spanning_tree_py_triangle_shortcut_prim",
        # Standard triangle (1, 2, 5): drop the heaviest edge.
        "origin": "constructed: triangle (1, 2, 5), Prim drops heaviest",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 5.0],
        "algo": "minimum_spanning_tree",
        "params": {"method": "prim"},
        "expected": {"total_weight": 3.0, "edge_count": 2},
    },
    {
        "case": "spanning_tree_py_disconnected_forest_automatic",
        # Two disconnected weighted edges; spanning forest = both edges.
        "origin": "constructed: 4-vertex two disjoint edges, "
        "automatic dispatch picks Kruskal",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [2.0, 3.0],
        "algo": "minimum_spanning_tree",
        "params": {"method": "automatic"},
        "expected": {"total_weight": 5.0, "edge_count": 2},
    },
]

# ALGO-GN-001: erdos_renyi_gnp / erdos_renyi_gnm. ER generators don't
# operate on an input graph — they produce one. Cross-implementation
# seed portability is impossible (each library uses its own RNG), so we
# capture **structural invariants** only:
#
#   * vcount: exact match with `params["n"]`
#   * ecount (gnp): inside a ±6σ band around µ = p · max_edges, where
#     ecount ~ Binomial(max_edges, p). 6σ gives a one-in-a-billion false
#     positive even at the band edges, well above CI flake tolerance.
#   * ecount (gnm): exact match with `params["m"]` — the algorithm
#     samples without replacement, so this is a sharp constraint.
#   * directed: exact boolean match.
#
# python-igraph's API for reference: `ig.Graph.Erdos_Renyi(n, p=p, m=m,
# directed=False, loops=False)`. We don't actually invoke it from this
# extractor — the seed is RNG-dependent so the captured ecount wouldn't
# be portable. Bands are hand-derived once per case and reviewed by the
# AWU author.
ERDOS_RENYI_GNP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "erdos_renyi_gnp_py_undirected_no_loops_n20_p05",
        # G(20, 0.5) undirected, no self-loops. max_edges = 20·19/2 = 190.
        # µ = 95, σ ≈ 6.89, ±6σ band ≈ [54, 136] → use [50, 140] for
        # safety.
        "origin": "constructed (mirrors ig.Graph.Erdos_Renyi(n=20, p=0.5, "
        "directed=False, loops=False)): expected µ=95 edges, "
        "Binomial(190, 0.5) ±6σ band",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 20,
            "p": 0.5,
            "directed": False,
            "loops": False,
            "seed": 12_345,
        },
        "expected": {
            "vcount": 20,
            "ecount_min": 50,
            "ecount_max": 140,
            "directed": False,
        },
    },
    {
        "case": "erdos_renyi_gnp_py_directed_no_loops_n15_p03",
        # Directed G(15, 0.3) without loops. max_edges = 15·14 = 210.
        # µ = 63, σ ≈ 6.64, ±6σ band ≈ [23, 103] → use [25, 100].
        "origin": "constructed (mirrors ig.Graph.Erdos_Renyi(n=15, p=0.3, "
        "directed=True, loops=False)): Binomial(210, 0.3) band",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 15,
            "p": 0.3,
            "directed": True,
            "loops": False,
            "seed": 4_242,
        },
        "expected": {
            "vcount": 15,
            "ecount_min": 25,
            "ecount_max": 100,
            "directed": True,
        },
    },
    {
        "case": "erdos_renyi_gnp_py_p0_no_edges_n10",
        # p = 0 → empty graph regardless of seed. Sharp test of the
        # boundary case.
        "origin": "constructed (mirrors ig.Graph.Erdos_Renyi(n=10, p=0.0)): "
        "p=0 forces empty edge set",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 10,
            "p": 0.0,
            "directed": False,
            "loops": False,
            "seed": 7,
        },
        "expected": {
            "vcount": 10,
            "ecount_min": 0,
            "ecount_max": 0,
            "directed": False,
        },
    },
    {
        "case": "erdos_renyi_gnp_py_p1_complete_n8",
        # p = 1 → complete graph (K8 with 28 edges). Sharp test on the
        # other boundary.
        "origin": "constructed (mirrors ig.Graph.Erdos_Renyi(n=8, p=1.0)): "
        "p=1 forces complete graph K8",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 8,
            "p": 1.0,
            "directed": False,
            "loops": False,
            "seed": 99,
        },
        "expected": {
            "vcount": 8,
            "ecount_min": 28,
            "ecount_max": 28,
            "directed": False,
        },
    },
]

ERDOS_RENYI_GNM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "erdos_renyi_gnm_py_undirected_n10_m15",
        # G(10, 15) undirected, no loops. Sampling without replacement
        # → ecount = 15 exactly.
        "origin": "constructed (mirrors ig.Graph.Erdos_Renyi(n=10, m=15)): "
        "uniform without-replacement sampling, ecount exact",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 10,
            "m": 15,
            "directed": False,
            "loops": False,
            "seed": 67_890,
        },
        "expected": {"vcount": 10, "ecount": 15, "directed": False},
    },
    {
        "case": "erdos_renyi_gnm_py_complete_n5_m10",
        # m == max_edges → must produce K5 every time, regardless of seed.
        "origin": "constructed (mirrors ig.Graph.Erdos_Renyi(n=5, m=10)): "
        "m equals max_edges, must be K5",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 5,
            "m": 10,
            "directed": False,
            "loops": False,
            "seed": 1_000,
        },
        "expected": {"vcount": 5, "ecount": 10, "directed": False},
    },
    {
        "case": "erdos_renyi_gnm_py_directed_n8_m20",
        # Directed G(8, 20). max_edges = 56, picks 20 ordered pairs.
        "origin": "constructed (mirrors ig.Graph.Erdos_Renyi(n=8, m=20, "
        "directed=True)): ordered pair sampling",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 8,
            "m": 20,
            "directed": True,
            "loops": False,
            "seed": 55_555,
        },
        "expected": {"vcount": 8, "ecount": 20, "directed": True},
    },
    {
        "case": "erdos_renyi_gnm_py_m0_no_edges_n12",
        # m = 0 → empty graph regardless of seed.
        "origin": "constructed (mirrors ig.Graph.Erdos_Renyi(n=12, m=0)): "
        "m=0 forces empty edge set",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 12,
            "m": 0,
            "directed": False,
            "loops": False,
            "seed": 31_415,
        },
        "expected": {"vcount": 12, "ecount": 0, "directed": False},
    },
]

# ALGO-GN-002: barabasi_game_bag. Like ER, this is a generator —
# cross-implementation seed portability is impossible, but the BAG
# variant happens to be **deterministic in edge count** when `m` is a
# constant: exactly `(n - 1) · m` edges. We also encode the BA temporal
# ordering invariant (`dst < src` for every edge) via a flag in
# `expected`. The conformance harness verifies these structurally for
# any seed.
#
# python-igraph reference API: `ig.Graph.Barabasi(n=n, m=m, power=1.0,
# outpref=outpref, directed=directed)`. Not invoked here — seed is
# RNG-dependent.
BARABASI_BAG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_game_bag_py_directed_n30_m2",
        "origin": "constructed (mirrors ig.Graph.Barabasi(n=30, m=2, "
        "outpref=False, directed=True)): edge count exact, BA "
        "temporal ordering",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 30,
            "m": 2,
            "outpref": False,
            "directed": True,
            "seed": 11_111,
        },
        "expected": {
            "vcount": 30,
            "ecount": 58,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "barabasi_game_bag_py_undirected_n40_m3",
        "origin": "constructed (mirrors ig.Graph.Barabasi(n=40, m=3, "
        "outpref=True, directed=False)): undirected forces outpref=True",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 40,
            "m": 3,
            "outpref": False,
            "directed": False,
            "seed": 22_222,
        },
        "expected": {
            "vcount": 40,
            "ecount": 117,
            "directed": False,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "barabasi_game_bag_py_directed_outpref_n20_m4",
        "origin": "constructed (mirrors ig.Graph.Barabasi(n=20, m=4, "
        "outpref=True, directed=True)): outpref biases on out-degree",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 20,
            "m": 4,
            "outpref": True,
            "directed": True,
            "seed": 33_333,
        },
        "expected": {
            "vcount": 20,
            "ecount": 76,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "barabasi_game_bag_py_m0_no_edges_n10",
        "origin": "constructed (mirrors ig.Graph.Barabasi(n=10, m=0)): "
        "m=0 yields n isolated vertices",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 10,
            "m": 0,
            "outpref": False,
            "directed": True,
            "seed": 44_444,
        },
        "expected": {
            "vcount": 10,
            "ecount": 0,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
]

# ALGO-GN-003: growing_random_game. Generator — seed portability is
# impossible, so we encode structural invariants only:
#   - vcount = n
#   - ecount = (n - 1) · m  (exact)
#   - directed matches the requested flag
#   - citation=true ⇒ BA-style temporal order (`dst < src` for directed,
#     `src != dst` for undirected since storage canonicalizes min/max)
#
# python-igraph reference API: `ig.Graph.Growing_Random(n=n, m=m,
# directed=directed, citation=citation)`. Not invoked here — seed is
# RNG-dependent.
GROWING_RANDOM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "growing_random_py_directed_citation_n50_m2",
        "origin": "constructed (mirrors ig.Graph.Growing_Random(n=50, m=2, "
        "directed=True, citation=True)): edge count exact, citation "
        "temporal ordering",
        "algo": "growing_random_game",
        "params": {
            "n": 50,
            "m": 2,
            "directed": True,
            "citation": True,
            "seed": 55_555,
        },
        "expected": {
            "vcount": 50,
            "ecount": 98,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "growing_random_py_directed_free_n40_m3",
        "origin": "constructed (mirrors ig.Graph.Growing_Random(n=40, m=3, "
        "directed=True, citation=False)): free-mode picks both endpoints",
        "algo": "growing_random_game",
        "params": {
            "n": 40,
            "m": 3,
            "directed": True,
            "citation": False,
            "seed": 66_666,
        },
        "expected": {
            "vcount": 40,
            "ecount": 117,
            "directed": True,
            "ba_temporal_order": False,
        },
    },
    {
        "case": "growing_random_py_undirected_citation_n30_m2",
        "origin": "constructed (mirrors ig.Graph.Growing_Random(n=30, m=2, "
        "directed=False, citation=True)): undirected citation",
        "algo": "growing_random_game",
        "params": {
            "n": 30,
            "m": 2,
            "directed": False,
            "citation": True,
            "seed": 77_777,
        },
        "expected": {
            "vcount": 30,
            "ecount": 58,
            "directed": False,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "growing_random_py_m0_no_edges_n15",
        "origin": "constructed (mirrors ig.Graph.Growing_Random(n=15, m=0)): "
        "m=0 yields n isolated vertices",
        "algo": "growing_random_game",
        "params": {
            "n": 15,
            "m": 0,
            "directed": True,
            "citation": True,
            "seed": 88_888,
        },
        "expected": {
            "vcount": 15,
            "ecount": 0,
            "directed": True,
            "ba_temporal_order": False,
        },
    },
]

# ALGO-GN-004: tree_game (LERW method). Generator — RNG state is not
# portable, so structural invariants only:
#   - vcount = n
#   - ecount = max(0, n - 1)  (exact spanning-tree edge count)
#   - directed matches the requested flag
#   - the edge set is a tree (acyclic + connected on the undirected
#     projection) — checked by union-find in the Rust harness
#
# python-igraph reference API: `ig.Graph.Tree_Game(n=n, directed=directed,
# method="lerw")`. Not invoked here — seed is RNG-dependent.
TREE_LERW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "tree_lerw_py_undirected_n10",
        "origin": "constructed (mirrors ig.Graph.Tree_Game(n=10, "
        "directed=False, method='lerw')): small undirected spanning tree",
        "algo": "tree_game_lerw",
        "params": {"n": 10, "directed": False, "seed": 101_010},
        "expected": {"vcount": 10, "ecount": 9, "directed": False, "is_tree": True},
    },
    {
        "case": "tree_lerw_py_undirected_n50",
        "origin": "constructed (mirrors ig.Graph.Tree_Game(n=50, "
        "directed=False, method='lerw')): medium undirected spanning tree",
        "algo": "tree_game_lerw",
        "params": {"n": 50, "directed": False, "seed": 202_020},
        "expected": {"vcount": 50, "ecount": 49, "directed": False, "is_tree": True},
    },
    {
        "case": "tree_lerw_py_directed_n30",
        "origin": "constructed (mirrors ig.Graph.Tree_Game(n=30, "
        "directed=True, method='lerw')): directed spanning tree, edges "
        "point parent→child in walk order",
        "algo": "tree_game_lerw",
        "params": {"n": 30, "directed": True, "seed": 303_030},
        "expected": {"vcount": 30, "ecount": 29, "directed": True, "is_tree": True},
    },
    {
        "case": "tree_lerw_py_n0_empty",
        "origin": "constructed: n=0 returns empty graph",
        "algo": "tree_game_lerw",
        "params": {"n": 0, "directed": False, "seed": 404_040},
        "expected": {"vcount": 0, "ecount": 0, "directed": False, "is_tree": False},
    },
]

# ALGO-GN-005: grg_game. python-igraph ships `ig.Graph.GRG(n, radius,
# torus=False, return_coordinates=False)`. RNG is not portable so we
# encode structural invariants only — vcount, undirected, simple, and a
# loose ±60 % band around the Poisson mean for the predicted edge count.
GRG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "grg_py_plane_n40_r025",
        "origin": "constructed (mirrors ig.Graph.GRG(n=40, radius=0.25, "
        "torus=False)): low-density disk graph",
        "algo": "grg_game",
        # n=40, r=0.25: predicted ≈ 153 edges; band [60, 250].
        "params": {"n": 40, "radius": 0.25, "torus": False, "seed": 6_660_001},
        "expected": {
            "vcount": 40,
            "directed": False,
            "is_simple": True,
            "ecount_min": 60,
            "ecount_max": 260,
        },
    },
    {
        "case": "grg_py_torus_n60_r018",
        "origin": "constructed (mirrors ig.Graph.GRG(n=60, radius=0.18, "
        "torus=True)): torus with wrap-around",
        "algo": "grg_game",
        # n=60, r=0.18: predicted ≈ 60·59/2 · π·0.0324 ≈ 180 edges.
        "params": {"n": 60, "radius": 0.18, "torus": True, "seed": 6_660_002},
        "expected": {
            "vcount": 60,
            "directed": False,
            "is_simple": True,
            "ecount_min": 70,
            "ecount_max": 290,
        },
    },
    {
        "case": "grg_py_dense_n25_r200",
        "origin": "constructed (mirrors ig.Graph.GRG(n=25, radius=2.0, "
        "torus=False)): radius > sqrt(2) yields complete graph",
        "algo": "grg_game",
        "params": {"n": 25, "radius": 2.0, "torus": False, "seed": 6_660_003},
        "expected": {
            "vcount": 25,
            "directed": False,
            "is_simple": True,
            "ecount_min": 300,  # 25*24/2 = 300 exactly
            "ecount_max": 300,
        },
    },
    {
        "case": "grg_py_singleton",
        "origin": "constructed: n=1 returns a singleton",
        "algo": "grg_game",
        "params": {"n": 1, "radius": 0.5, "torus": False, "seed": 6_660_004},
        "expected": {
            "vcount": 1,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
]

# ALGO-FL-002: max_flow_value. Mirrors `g.maxflow_value(source, target,
# capacities)` from python-igraph (Cython wrapper on the same
# `igraph_maxflow_value` C entry point). The python-igraph test file
# tests/test_flow.py:36-40 builds the same 4-vertex undirected graph
# used by the C unit test and asserts the unit / weighted max-flow
# values directly; we replay the same two assertions.
MAXFLOW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "maxflow_py_undirected_4v_unit",
        "origin": "python-igraph tests/test_flow.py:MaxFlowTests.testMaxFlowValue "
        "(g = Graph(4, [(0,1),(0,2),(1,2),(1,3),(2,3)]); "
        "g.maxflow_value(0, 3) == 2)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "algo": "max_flow_value",
        "params": {"source": 0, "target": 3, "use_capacity": False},
        "expected": 2.0,
    },
    {
        "case": "maxflow_py_undirected_4v_weighted",
        "origin": "python-igraph tests/test_flow.py:MaxFlowTests.testMaxFlowValue "
        "(g.maxflow_value(0, 3, [4, 2, 10, 2, 2]) == 4)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "graph_weights": [4.0, 2.0, 10.0, 2.0, 2.0],
        "algo": "max_flow_value",
        "params": {"source": 0, "target": 3, "use_capacity": True},
        "expected": 4.0,
    },
]

# ALGO-FL-010: st_mincut_value. python-igraph exposes
# `Graph.mincut_value(source, target, capacity)`. Test
# tests/test_flow.py:MinCutTests.testMinCutValue:72-80 asserts
# `g.mincut_value(0, 3) == 2` (unit caps) and
# `g.mincut_value(0, 3, [4,2,10,2,2]) == 4` (weighted) on the same
# 4-vertex undirected graph used for maxflow. We replay both
# assertions verbatim — they hit the same `igraph_st_mincut_value` C
# entry point that the C and R sources do.
ST_MINCUT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_mincut_py_undirected_4v_unit",
        "origin": "python-igraph tests/test_flow.py:MinCutTests.testMinCutValue:74 "
        "(g = Graph(4, [(0,1),(0,2),(1,2),(1,3),(2,3)]); "
        "g.mincut_value(0, 3) == 2)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "algo": "st_mincut_value",
        "params": {"source": 0, "target": 3, "use_capacity": False},
        "expected": 2.0,
    },
    {
        "case": "st_mincut_py_undirected_4v_weighted",
        "origin": "python-igraph tests/test_flow.py:MinCutTests.testMinCutValue:75 "
        "(g.mincut_value(0, 3, [4, 2, 10, 2, 2]) == 4)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "graph_weights": [4.0, 2.0, 10.0, 2.0, 2.0],
        "algo": "st_mincut_value",
        "params": {"source": 0, "target": 3, "use_capacity": True},
        "expected": 4.0,
    },
]


# ALGO-FL-011: st_edge_connectivity. python-igraph exposes
# `Graph.edge_connectivity(source, target)` which dispatches to
# `igraph_st_edge_connectivity` when both endpoints are supplied. Test
# tests/test_flow.py:CutTests.testEdgeConnectivity:18 asserts
# `g.edge_connectivity(0, 3) == 2` on the same 4-vertex undirected
# graph used for maxflow. We replay that assertion verbatim plus a
# directed K4 sanity check (every pair has ec = 3 since every vertex
# has out-degree 3 to every other vertex).
ST_EDGE_CONN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_edge_conn_py_undirected_4v",
        "origin": "python-igraph tests/test_flow.py:CutTests.testEdgeConnectivity:18 "
        "(g = Graph(4, [(0,1),(0,2),(1,2),(1,3),(2,3)]); "
        "g.edge_connectivity(0, 3) == 2)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "algo": "st_edge_connectivity",
        "params": {"source": 0, "target": 3},
        "expected": 2,
    },
    {
        "case": "st_edge_conn_py_undirected_full_5v",
        "origin": "K_5 undirected: every pair (s, t) has 4 edge-disjoint "
        "paths → st_edge_connectivity(s, t) == 4 (matches "
        "edge_connectivity(K_5) == 4 in test-flow.R:148)",
        "graph_factory": lambda: ig.Graph.Full(5, directed=False),
        "algo": "st_edge_connectivity",
        "params": {"source": 0, "target": 4},
        "expected": 4,
    },
]


# ALGO-FL-012: edge_disjoint_paths. python-igraph aliases
# `Graph.edge_disjoint_paths = Graph.edge_connectivity` in
# src/igraph/__init__.py:342, so the same `CutTests.testEdgeConnectivity`
# fixture from tests/test_flow.py:18 exercises both names. Documented as
# a Menger-theorem equivalent in doc/source/analysis.rst:204. Replays the
# 4-vertex undirected fixture (ec == 2) plus a K_5 sanity (ep == 4).
ED_PATHS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_disjoint_paths_py_undirected_4v",
        "origin": "python-igraph tests/test_flow.py:CutTests.testEdgeConnectivity:18 "
        "(g = Graph(4, [(0,1),(0,2),(1,2),(1,3),(2,3)]); "
        "g.edge_disjoint_paths(0, 3) == 2 via alias "
        "edge_disjoint_paths = edge_connectivity at "
        "src/igraph/__init__.py:342)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "algo": "edge_disjoint_paths",
        "params": {"source": 0, "target": 3},
        "expected": 2,
    },
    {
        "case": "edge_disjoint_paths_py_undirected_full_5v",
        "origin": "K_5 undirected: every vertex has 4 incident edges so by "
        "Menger ep(s, t) == 4 for every pair; matches "
        "edge_connectivity(K_5) == 4 in test-flow.R:148",
        "graph_factory": lambda: ig.Graph.Full(5, directed=False),
        "algo": "edge_disjoint_paths",
        "params": {"source": 0, "target": 4},
        "expected": 4,
    },
]


# ALGO-FL-013: st_vertex_connectivity. python-igraph exposes the same
# function (`Graph.vertex_connectivity(source, target, neighbors=...)`)
# from src/igraph/__init__.py. The `tests/test_flow.py`
# `CutTests.testVertexConnectivity` test verifies the same 4-vertex
# fixture as testEdgeConnectivity: G = (0,1)(0,2)(1,2)(1,3)(2,3) →
# vc(0, 3) == 2 (vertices 1 and 2 are both bottlenecks; either alone
# suffices to disconnect → vc = 2). Plus a K_5 directed sanity.
ST_VCONN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_vconn_py_undirected_4v_error",
        "origin": "python-igraph tests/test_flow.py:CutTests.testVertexConnectivity "
        "(g = Graph(4, [(0,1),(0,2),(1,2),(1,3),(2,3)]); "
        "g.vertex_connectivity(0, 3, neighbors='error') == 2 — no direct "
        "edge between 0 and 3 so ERROR mode is safe; vertex 1 and 2 each "
        "lie on every 0→3 path so vc = 2)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 3, "mode": "error"},
        "expected": 2,
    },
    {
        "case": "st_vconn_py_full_5v_directed_ignore",
        "origin": "K_5 directed (both arcs for every pair). Every pair has "
        "n-2 = 3 internally vertex-disjoint paths plus a direct arc; "
        "with IGNORE mode the direct arc is subtracted → vc = 3",
        "graph_factory": lambda: ig.Graph.Full(5, directed=True, loops=False),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "ignore"},
        "expected": 3,
    },
]


# ALGO-FL-014: vertex_disjoint_paths. python-igraph aliases
# `Graph.vertex_disjoint_paths` to `Graph.vertex_connectivity(source,
# target)` at src/igraph/__init__.py:341 — the dedicated `igraph_vertex_
# disjoint_paths` C entry is exposed without the explicit `neighbors=`
# parameter (the C implementation always uses `IGRAPH_VCONN_NEI_IGNORE`
# and adds the direct-edge count). Two fixtures echo the rigraph
# test-flow.R:202-206 cases, giving Rust port + python + R triple
# cross-validation on the same minimal graphs.
VDP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "vdp_py_full_5v_directed_0_to_1",
        "origin": "K_5 directed (every pair has both arcs); "
        "vertex_disjoint_paths(0, 1) = n-1 = 4 (direct arc + "
        "3 disjoint detours through {2,3,4})",
        "graph_factory": lambda: ig.Graph.Full(5, directed=True, loops=False),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 0, "target": 1},
        "expected": 4,
    },
    {
        "case": "vdp_py_path_undirected_0_to_3",
        "origin": "Undirected path 0-1-2-3-4; vertex_disjoint_paths(0, 3) "
        "= 1 because vertices 1 and 2 are bottlenecks for any 0→3 walk",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 0, "target": 3},
        "expected": 1,
    },
]


# ALGO-FL-015: global vertex_connectivity (cohesion). Mirrors
# `Graph.vertex_connectivity()` (no source/target) and `Graph.cohesion()`
# in python-igraph (Cython wrapper on `igraph_vertex_connectivity`).
# Two fixtures: a Barabasi tree (vc = 1) and a directed BFS in-tree
# (vc = 0, not strongly connected) — both straight from
# test_flow.py:27-30.
VCONN_GLOBAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "vconn_py_full_4v_returns_three",
        "origin": "K_4 undirected; complete graph short-circuit yields "
        "vc = vcount - 1 = 3 (test_flow.py:CutTests setUp uses similar "
        "small fixtures)",
        "graph_factory": lambda: ig.Graph.Full(4, directed=False, loops=False),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 3,
    },
    {
        "case": "vconn_py_tree_10v_undirected_returns_one",
        "origin": "test_flow.py:29 — Graph.Tree(10, 3).cohesion() == 1 "
        "(undirected balanced ternary tree of 10 nodes; every leaf has "
        "degree 1 so vc = 1 via min-degree short-circuit)",
        "graph_factory": lambda: ig.Graph.Tree(10, 3),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 1,
    },
]


# ALGO-FL-016: global edge_connectivity (adhesion). Mirrors
# `Graph.edge_connectivity()` (no source/target) and `Graph.adhesion()`
# in python-igraph (Cython wrapper on `igraph_edge_connectivity`).
# Two fixtures exercise the two main paths: a complete undirected K_4
# (no cheap shortcut — completeness alone doesn't bound edge connectivity
# for multigraphs — so the fixed-vertex loop runs and yields n-1 = 3),
# plus a directed BFS in-tree (not strongly connected ⇒ 0 via cheap
# connectedness check).
ECONN_GLOBAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "econn_py_full_4v_returns_three",
        "origin": "K_4 undirected; no complete-graph short-circuit for "
        "edge_connectivity (flow.c:2168-2180 comment), so the fixed-vertex "
        "loop runs and returns n-1 = 3 (min cut isolates any single vertex)",
        "graph_factory": lambda: ig.Graph.Full(4, directed=False, loops=False),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 3,
    },
    {
        "case": "econn_py_tree_10v_directed_returns_zero",
        "origin": "test_flow.py:27 — Graph.Tree(10, 3, mode='out').adhesion() "
        "== 0 (directed out-tree of 10 nodes is not strongly connected, so the "
        "cheap connectedness check short-circuits to 0)",
        "graph_factory": lambda: ig.Graph.Tree(10, 3, mode="out"),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 0,
    },
]


# ALGO-FL-017: mincut_value — weighted global minimum-cut value.
# python-igraph exposes `Graph.mincut_value(capacity=None)` directly
# (test_flow.py:CutTests.testMincutValue). We pin two cases: a
# unit-capacity ring (matches edge_connectivity) and a weighted
# undirected path where the bottleneck edge dominates the cut.
MINCUT_VALUE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mincut_py_ring5_undirected_unit_caps_returns_two",
        "origin": "test_flow.py — Graph.Ring(5).mincut_value() == 2.0; "
        "unit-capacity ring lambda(C_5) = 2 (mirrors edge_connectivity).",
        "graph_factory": lambda: ig.Graph.Ring(5, directed=False, circular=True),
        "algo": "mincut_value",
        "params": {"capacity": None},
        "expected": 2.0,
    },
    {
        "case": "mincut_py_path5_undirected_weighted_returns_one_quarter",
        "origin": "Weighted undirected path 0-1-2-3-4 with capacities "
        "[1, 1, 0.25, 1]; bridge edge 2-3 has the smallest capacity, "
        "so the global min cut isolates {3, 4} at cost 0.25.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 0.25, 1.0],
        "algo": "mincut_value",
        "params": {"capacity": [1.0, 1.0, 0.25, 1.0]},
        "expected": 0.25,
    },
]

# ALGO-FL-018: st_mincut (full s-t partition). python-igraph exposes
# `Graph.st_mincut(source, target, capacity=None)` which returns a Cut
# object carrying .value, .cut (edge ids), .partition[0] / partition[1].
# We pin three regimes: (1) parallel arcs multigraph — unique min cut
# saturates both, (2) directed bottleneck — pin every field, (3)
# disconnected endpoints — value 0, empty cut, partition = {source}.
ST_MINCUT_PARTITION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_mincut_py_multigraph_two_parallel_arcs",
        "origin": "Two parallel arcs 0→1 (directed multigraph). "
        "Graph.st_mincut(0, 1) ⇒ value=2, cut=[0,1], partition=[0], "
        "partition2=[1] — both arcs sit on the only frontier from 0.",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1)], directed=True
        ),
        "algo": "st_mincut",
        "params": {"source": 0, "target": 1, "capacity": None},
        "expected": {
            "value": 2.0,
            "cut": [0, 1],
            "partition": [0],
            "partition2": [1],
        },
    },
    {
        "case": "st_mincut_py_directed_bottleneck_weighted",
        "origin": "Graph(n=4, edges=[(0,1),(1,2),(2,3)], directed=True) "
        "with capacity [5,2,7]; unique bottleneck arc (1,2) cap 2 ⇒ "
        "Graph.st_mincut(0, 3) value=2, cut=[1], partition=[0,1], "
        "partition2=[2,3].",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "graph_weights": [5.0, 2.0, 7.0],
        "algo": "st_mincut",
        "params": {"source": 0, "target": 3, "capacity": [5.0, 2.0, 7.0]},
        "expected": {
            "value": 2.0,
            "cut": [1],
            "partition": [0, 1],
            "partition2": [2, 3],
        },
    },
    {
        "case": "st_mincut_py_disconnected_zero_value",
        "origin": "Graph(n=4) with no edges. Graph.st_mincut(0, 3) ⇒ "
        "value=0, empty cut, partition={0}, partition2={1,2,3}: "
        "no path from source to target, so the empty cut suffices.",
        "graph_factory": lambda: ig.Graph(n=4, edges=[], directed=True),
        "algo": "st_mincut",
        "params": {"source": 0, "target": 3, "capacity": None},
        "expected": {
            "value": 0.0,
            "cut": [],
            "partition": [0],
            "partition2": [1, 2, 3],
        },
    },
]

# ALGO-FL-020: gomory_hu_tree. python-igraph exposes
# `Graph.gomory_hu_tree(capacity=None, flow="flow")` which returns a
# tree Graph with edge attribute "flow" holding the per-edge min-cut
# weights. Since the tree shape is not unique (Gusfield depends on
# scan order), we pin only shape invariants here; the runner verifies
# the Gomory-Hu property by recomputing `max_flow_value` for every
# pair and asserting equality with the min-edge-weight along the
# unique tree path between them. Fixtures cover (1) the python-igraph
# tutorial 4-vertex path with non-uniform caps, (2) a 5-vertex cycle
# with unit caps, (3) the C-suite 6v weighted case (so all three
# extractors share a reference fixture).
GOMORY_HU_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "gomory_hu_py_path4_nonuniform_caps",
        "origin": "Graph(n=4, edges=[(0,1),(1,2),(2,3)], directed=False) "
        "with capacity [3,1,5]; the (1,2) bridge of cap 1.0 dominates "
        "every pair crossing it, so the GH tree carries weight 1.0 on "
        "at least one edge.",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [3.0, 1.0, 5.0],
        "algo": "gomory_hu_tree",
        "params": {"capacity": [3.0, 1.0, 5.0]},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "flows_len": 3,
            "flows_min": 1.0,
            "is_directed": False,
        },
    },
    {
        "case": "gomory_hu_py_cycle5_unit_caps",
        "origin": "C_5 undirected unit caps (5-vertex cycle). Every "
        "pair has max-flow exactly 2 (two edge-disjoint paths around "
        "the cycle), so every GH tree edge weight = 2.",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)],
            directed=False,
        ),
        "algo": "gomory_hu_tree",
        "params": {"capacity": None},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "flows_len": 4,
            "flows_min": 2.0,
            "is_directed": False,
        },
    },
    {
        "case": "gomory_hu_py_6v_weighted_shared_with_c",
        "origin": "Same fixture as C unit test (6-vertex undirected, "
        "caps [1,7,1,3,2,4,1,6,2]) but verified via python-igraph "
        "Graph.gomory_hu_tree(). The min cut across the entire graph "
        "is 5 (cut {0,2} from {1,3,4,5} sums caps 1+1+4=6 → not min; "
        "the actual global min cut sums to 5), so flows_min ≥ 0.",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1),
                (0, 2),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 4),
                (3, 4),
                (3, 5),
                (4, 5),
            ],
            directed=False,
        ),
        "graph_weights": [1.0, 7.0, 1.0, 3.0, 2.0, 4.0, 1.0, 6.0, 2.0],
        "algo": "gomory_hu_tree",
        "params": {"capacity": [1.0, 7.0, 1.0, 3.0, 2.0, 4.0, 1.0, 6.0, 2.0]},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "flows_len": 5,
            "flows_min": 0.0,
            "is_directed": False,
        },
    },
    {
        "case": "gomory_hu_py_directed_rejects",
        "origin": "python-igraph Graph.gomory_hu_tree on a directed "
        "graph raises InternalError (igraph C returns IGRAPH_EINVAL).",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "algo": "gomory_hu_tree",
        "params": {"capacity": None},
        "expected": {"raises": True},
    },
]


# ALGO-FL-030: dominator_tree. python-igraph exposes
# `Graph.dominator(root, mode=)` (note the singular name — `dominator`,
# not `dominator_tree`), which returns a Python list of immediate-dominator
# vertex ids with `-1` at the root and `float('nan')` at unreachable
# vertices (python-igraph upcasts to float for the NaN sentinel). The
# fixtures below mirror python-igraph/tests/test_structural.py:1057-1175
# `StructuralTests.testDominators` exactly. We normalise the float NaN
# to the integer sentinel `-2` in the `expected.idom` JSON, matching the
# Rust port's `DominatorTree { idom: Vec<i32> }` convention (root = -1,
# unreachable = -2). The runner uses element-wise int comparison.
DOMINATOR_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dominator_py_13v_classical_out",
        "origin": "python-igraph/tests/test_structural.py:1057-1090 — "
        "13-vertex Lengauer-Tarjan example, g.dominator(0).",
        "graph_factory": lambda: ig.Graph(
            n=13,
            edges=[
                (0, 1), (0, 7), (0, 10),
                (1, 2), (1, 5),
                (2, 3),
                (3, 4),
                (4, 3), (4, 0),
                (5, 3), (5, 6),
                (6, 3),
                (7, 8), (7, 10), (7, 11),
                (8, 9),
                (9, 4), (9, 8),
                (10, 11),
                (11, 12),
                (12, 9),
            ],
            directed=True,
        ),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "out"},
        "expected": {
            "idom": [-1, 0, 1, 0, 0, 1, 5, 0, 0, 0, 0, 0, 11],
            "leftout": [],
        },
    },
    {
        "case": "dominator_py_13v_reversed_in",
        "origin": "python-igraph/tests/test_structural.py:1092-1122 — "
        "13-vertex flowgraph with reversed edges, g.dominator(0, mode=IN).",
        "graph_factory": lambda: ig.Graph(
            n=13,
            edges=[
                (1, 0), (2, 0), (3, 0),
                (4, 1),
                (1, 2), (4, 2), (5, 2),
                (6, 3), (7, 3),
                (12, 4),
                (8, 5),
                (9, 6),
                (9, 7), (10, 7),
                (5, 8), (11, 8),
                (11, 9),
                (9, 10),
                (9, 11), (0, 11),
                (8, 12),
            ],
            directed=True,
        ),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "in"},
        "expected": {
            "idom": [-1, 0, 0, 0, 0, 0, 3, 3, 0, 0, 7, 0, 4],
            "leftout": [],
        },
    },
    {
        "case": "dominator_py_20v_unreachable_out",
        "origin": "python-igraph/tests/test_structural.py:1124-1175 — "
        "20-vertex graph with unreachable component {5,6,7,16..19}; "
        "Python NaN sentinels normalised to -2 for the Rust runner.",
        "graph_factory": lambda: ig.Graph(
            n=20,
            edges=[
                (0, 1), (0, 2), (0, 3),
                (1, 4),
                (2, 1), (2, 4), (2, 8),
                (3, 9), (3, 10),
                (4, 15),
                (8, 11),
                (9, 12),
                (10, 12), (10, 13),
                (11, 8), (11, 14),
                (12, 14),
                (13, 12),
                (14, 12), (14, 0),
                (15, 11),
            ],
            directed=True,
        ),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "out"},
        "expected": {
            "idom": [
                -1, 0, 0, 0, 0, -2, -2, -2, 0, 3,
                3, 0, 0, 10, 0, 4, -2, -2, -2, -2,
            ],
            "leftout": [5, 6, 7, 16, 17, 18, 19],
        },
    },
    {
        "case": "dominator_py_undirected_rejects",
        "origin": "python-igraph Graph.dominator on an undirected graph "
        "raises InternalError (igraph C returns IGRAPH_EINVAL — the "
        "algorithm is defined only for directed flowgraphs).",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "out"},
        "expected": {"raises": True},
    },
]


# ALGO-GN-006: forest_fire_game. Mirrors `ig.Graph.Forest_Fire(n,
# fw_prob, bw_factor, ambs, directed)` from python-igraph (Cython
# wrapper on the same `igraph_forest_fire_game` C entry point). RNG
# state is not portable, so we encode structural invariants only —
# vcount, directed, is_simple (no loops + no parallels), and a loose
# ecount band anchored on the per-actnode lower bound (n-1 when ambs > 0)
# and a generous upper band tolerant of burn-tail variance.
FOREST_FIRE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "forest_fire_py_directed_n100_fw015_bw03_ambs2",
        "origin": "constructed (mirrors ig.Graph.Forest_Fire(n=100, "
        "fw_prob=0.15, bw_factor=0.3, ambs=2, directed=True)): "
        "moderate-burn directed graph",
        "algo": "forest_fire_game",
        "params": {
            "n": 100,
            "fw_prob": 0.15,
            "bw_factor": 0.3,
            "ambs": 2,
            "directed": True,
            "seed": 7_770_001,
        },
        "expected": {
            "vcount": 100,
            "directed": True,
            "is_simple": True,
            "ecount_min": 99,
            "ecount_max": 10000,
        },
    },
    {
        "case": "forest_fire_py_undirected_n60_fw025_bw04_ambs3",
        "origin": "constructed (mirrors ig.Graph.Forest_Fire(n=60, "
        "fw_prob=0.25, bw_factor=0.4, ambs=3, directed=False)): "
        "warmer burn with three ambassadors",
        "algo": "forest_fire_game",
        "params": {
            "n": 60,
            "fw_prob": 0.25,
            "bw_factor": 0.4,
            "ambs": 3,
            "directed": False,
            "seed": 7_770_002,
        },
        "expected": {
            "vcount": 60,
            "directed": False,
            "is_simple": True,
            "ecount_min": 59,
            "ecount_max": 6000,
        },
    },
    {
        "case": "forest_fire_py_n1_singleton",
        "origin": "constructed (mirrors ig.Graph.Forest_Fire boundary n=1): "
        "singleton has no edges regardless of burn params",
        "algo": "forest_fire_game",
        "params": {
            "n": 1,
            "fw_prob": 0.3,
            "bw_factor": 0.5,
            "ambs": 2,
            "directed": True,
            "seed": 7_770_003,
        },
        "expected": {
            "vcount": 1,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
]

# ALGO-GN-014: preference_game. Mirrors ig.Graph.Preference(n, type_dist,
# pref_matrix, ...) — Cython wrapper on `igraph_preference_game`. RNG
# state is not portable across implementations, so we capture
# structural invariants only.
PREFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "preference_py_two_blocks_diag_p1",
        "origin": "tests/test_games.py::testPreference — "
        "Graph.Preference(100, [1,1], [[1,0],[0,1]]) yields exactly 2 "
        "connected components (each a complete graph on ~50 vertices)",
        "algo": "preference_game",
        "params": {
            "nodes": 100,
            "types": 2,
            "type_dist": [1.0, 1.0],
            "fixed_sizes": False,
            "pref_matrix": [
                [1.0, 0.0],
                [0.0, 1.0],
            ],
            "directed": False,
            "loops": False,
            "seed": 9_990_001,
        },
        "expected": {
            "vcount": 100,
            "directed": False,
            "is_simple": True,
            # Two K_~50 cliques: each ~50*49/2 ≈ 1225, total ≈ 2450.
            # Use a generous band tolerant to type-balance variance.
            "ecount_min": 1_700,
            "ecount_max": 2_700,
            "diagonal_only_pref": True,
            "max_type": 1,
        },
    },
    {
        "case": "preference_py_three_blocks_diag_sparse",
        "origin": "constructed (mirrors ig.Graph.Preference): three "
        "balanced blocks at p=0.4 within, 0 across",
        "algo": "preference_game",
        "params": {
            "nodes": 60,
            "types": 3,
            "type_dist": None,
            "fixed_sizes": True,
            "pref_matrix": [
                [0.4, 0.0, 0.0],
                [0.0, 0.4, 0.0],
                [0.0, 0.0, 0.4],
            ],
            "directed": False,
            "loops": False,
            "seed": 9_990_002,
        },
        "expected": {
            "vcount": 60,
            "directed": False,
            "is_simple": True,
            # 3 blocks of size 20: 3 * C(20,2) * 0.4 ≈ 228; band ±50%.
            "ecount_min": 130,
            "ecount_max": 340,
            "diagonal_only_pref": True,
            "max_type": 2,
        },
    },
    {
        "case": "preference_py_zero_pref_isolates",
        "origin": "constructed (mirrors ig.Graph.Preference with "
        "pref_matrix all zero): vcount preserved, edgeless",
        "algo": "preference_game",
        "params": {
            "nodes": 30,
            "types": 2,
            "type_dist": [1.0, 1.0],
            "fixed_sizes": False,
            "pref_matrix": [
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            "directed": False,
            "loops": False,
            "seed": 9_990_003,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
            "diagonal_only_pref": False,
            "max_type": 1,
        },
    },
]

# ALGO-GN-014: asymmetric_preference_game. Mirrors
# ig.Graph.Asymmetric_Preference(n, type_dist_matrix, pref_matrix, ...).
ASYMMETRIC_PREFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "asym_preference_py_two_components",
        "origin": "tests/test_games.py::testAsymmetricPreference — "
        "Graph.Asymmetric_Preference(100, [[0,1],[1,0]], [[0,1],[1,0]]) "
        "yields 2 weakly connected components (vertices with same "
        "out_type/in_type pair stay disjoint)",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 100,
            "no_out_types": 2,
            "no_in_types": 2,
            "type_dist_matrix": [
                [0.0, 1.0],
                [1.0, 0.0],
            ],
            "pref_matrix": [
                [0.0, 1.0],
                [1.0, 0.0],
            ],
            "loops": False,
            "seed": 9_991_001,
        },
        "expected": {
            "vcount": 100,
            "directed": True,
            "is_simple": True,
            # Vertices split into two halves with (out=0,in=1) and
            # (out=1,in=0). Cross edges populate (0,1) and (1,0)
            # cells at p=1, intra-cell at p=0. Approx 100*100/2 ≈ 5000
            # off-diag cell rich; conservative band.
            "ecount_min": 4_500,
            "ecount_max": 5_500,
            "max_out_type": 1,
            "max_in_type": 1,
        },
    },
    {
        "case": "asym_preference_py_balanced_diag",
        "origin": "constructed (mirrors Graph.Asymmetric_Preference): "
        "joint type_dist diagonal so out_type==in_type for every "
        "vertex; pref_matrix diagonal at p=0.6",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 40,
            "no_out_types": 2,
            "no_in_types": 2,
            "type_dist_matrix": [
                [1.0, 0.0],
                [0.0, 1.0],
            ],
            "pref_matrix": [
                [0.6, 0.0],
                [0.0, 0.6],
            ],
            "loops": False,
            "seed": 9_991_002,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "is_simple": True,
            # 2 balanced blocks of size ~20; each at p=0.6 over 20*20-20
            # off-diag slots ≈ 0.6*380 ≈ 228; total ≈ 456.
            "ecount_min": 250,
            "ecount_max": 600,
            "max_out_type": 1,
            "max_in_type": 1,
        },
    },
    {
        "case": "asym_preference_py_zero_pref_edgeless",
        "origin": "constructed (mirrors Graph.Asymmetric_Preference "
        "with pref_matrix all zero): edgeless, types in range",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 25,
            "no_out_types": 2,
            "no_in_types": 3,
            "type_dist_matrix": None,
            "pref_matrix": [
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            "loops": False,
            "seed": 9_991_003,
        },
        "expected": {
            "vcount": 25,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
            "max_out_type": 1,
            "max_in_type": 2,
        },
    },
]

# ALGO-GN-015: establishment_game. Mirrors ig.Graph.Establishment(
# n, k, type_dist, pref_matrix, ...) — Cython wrapper on
# `igraph_establishment_game`. RNG state is not portable across
# implementations; structural invariants only.
ESTABLISHMENT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "establishment_py_uniform_full_p1_n40_2types_k3",
        "origin": "tests/test_games.py::testEstablishment — "
        "Graph.Establishment(n=40, k=3, type_dist=None, "
        "pref_matrix=ones(2,2)): every Floyd pick accepts ⇒ "
        "exactly (n-k)*k = 111 edges",
        "algo": "establishment_game",
        "params": {
            "nodes": 40,
            "types": 2,
            "k": 3,
            "type_dist": None,
            "pref_matrix": [
                [1.0, 1.0],
                [1.0, 1.0],
            ],
            "directed": False,
            "seed": 9_992_001,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "is_simple": True,
            "ecount_min": 111,
            "ecount_max": 111,
            "max_type": 1,
        },
    },
    {
        "case": "establishment_py_diag_only_n60_3types_k4",
        "origin": "constructed (mirrors Graph.Establishment): three types "
        "at uniform mass, diagonal pref 0.5; edges stay within types",
        "algo": "establishment_game",
        "params": {
            "nodes": 60,
            "types": 3,
            "k": 4,
            "type_dist": [1.0, 1.0, 1.0],
            "pref_matrix": [
                [0.5, 0.0, 0.0],
                [0.0, 0.5, 0.0],
                [0.0, 0.0, 0.5],
            ],
            "directed": False,
            "seed": 9_992_002,
        },
        "expected": {
            "vcount": 60,
            "directed": False,
            "is_simple": True,
            # Each step accepts diagonal candidates only when same-type
            # neighbour is sampled. Pr[same type] = 1/3 at uniform; expected
            # edges ≈ (60-4)*4 * (1/3) * 0.5 ≈ 37; band wide.
            "ecount_min": 12,
            "ecount_max": 80,
            "diagonal_only_pref": True,
            "max_type": 2,
        },
    },
    {
        "case": "establishment_py_zero_pref_edgeless_n30",
        "origin": "constructed (mirrors Graph.Establishment with pref=0): "
        "isolated vertices, types still assigned",
        "algo": "establishment_game",
        "params": {
            "nodes": 30,
            "types": 2,
            "k": 5,
            "type_dist": None,
            "pref_matrix": [
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            "directed": False,
            "seed": 9_992_003,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
            "max_type": 1,
        },
    },
]

# ALGO-GN-016: callaway_traits_game. Mirrors ig.Graph.Callaway_Traits(
# nodes, types, edges_per_step, type_dist, pref_matrix, directed,
# attribute) — Cython wrapper on `igraph_callaway_traits_game`. RNG
# state is not portable; structural invariants only. Note: candidate
# edges may include self-loops and parallel edges — output is NOT
# simple-by-construction (unlike establishment).
CALLAWAY_TRAITS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "callaway_py_uniform_full_p1_n35_2types_eps3",
        "origin": "tests/test_games.py::testCallawayTraits — "
        "Graph.Callaway_Traits(n=35, types=2, edges_per_step=3, "
        "type_dist=None, pref=ones, undirected): every candidate "
        "accepts ⇒ exactly (n-1)*eps = 102 edges",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 35,
            "types": 2,
            "edges_per_step": 3,
            "type_dist": None,
            "pref_matrix": [
                [1.0, 1.0],
                [1.0, 1.0],
            ],
            "directed": False,
            "seed": 9_992_011,
        },
        "expected": {
            "vcount": 35,
            "directed": False,
            "ecount_min": 102,  # (35-1)*3 = 102
            "ecount_max": 102,
            "max_type": 1,
        },
    },
    {
        "case": "callaway_py_diag_only_n50_3types_eps2",
        "origin": "constructed (mirrors Graph.Callaway_Traits): three "
        "types at uniform mass, diagonal pref at 0.5; accepted edges "
        "share endpoint type",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 50,
            "types": 3,
            "edges_per_step": 2,
            "type_dist": [1.0, 1.0, 1.0],
            "pref_matrix": [
                [0.5, 0.0, 0.0],
                [0.0, 0.5, 0.0],
                [0.0, 0.0, 0.5],
            ],
            "directed": False,
            "seed": 9_992_012,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            # Pr[same type] = 1/3; accept = 0.5 ⇒ E[ecount] ≈ 98 * 1/3 * 0.5 ≈ 16.
            "ecount_min": 4,
            "ecount_max": 50,
            "diagonal_only_pref": True,
            "max_type": 2,
        },
    },
    {
        "case": "callaway_py_zero_pref_edgeless_n40",
        "origin": "constructed (mirrors Graph.Callaway_Traits with pref=0): "
        "isolated vertices, types still assigned",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 40,
            "types": 2,
            "edges_per_step": 5,
            "type_dist": None,
            "pref_matrix": [
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            "directed": False,
            "seed": 9_992_013,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "ecount_min": 0,
            "ecount_max": 0,
            "max_type": 1,
        },
    },
]

# ALGO-GN-017: cited_type_game. Mirrors ig.Graph.Cited_Type(
# types, pref, edges_per_step, directed, ...) — Cython wrapper on
# `igraph_cited_type_game`. Vertex types are PRE-ASSIGNED by the caller
# (not sampled); each new vertex i ∈ [1, nodes) adds eps citations to
# previously-added vertices weighted by pref[type[v]]. RNG state is not
# portable; structural invariants only. Multi-edges allowed when eps≥2;
# self-loops only via sum=0 fallback.
CITED_TYPE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "cited_type_py_uniform_pref_n25_2types_eps2",
        "origin": "tests/test_games.py-style — Graph.Cited_Type(types=[0,1,...], "
        "pref=[1.0,1.0], edges_per_step=2, directed): uniform pref ⇒ "
        "exactly (n-1)*eps = 48 edges, no self-loops",
        "algo": "cited_type_game",
        "params": {
            "nodes": 25,
            "types": [v % 2 for v in range(25)],
            "pref": [1.0, 1.0],
            "edges_per_step": 2,
            "directed": True,
            "seed": 9_993_011,
        },
        "expected": {
            "vcount": 25,
            "directed": True,
            "ecount_min": 48,
            "ecount_max": 48,
            "no_self_loops": True,
            "max_type": 1,
        },
    },
    {
        "case": "cited_type_py_skewed_pref_n40_3types_eps3_undirected",
        "origin": "constructed (Graph.Cited_Type): three types with "
        "highly skewed pref=[5.0, 0.5, 0.01]; positive pref ⇒ no "
        "self-loops, exactly (n-1)*eps = 117 edges",
        "algo": "cited_type_game",
        "params": {
            "nodes": 40,
            "types": [v % 3 for v in range(40)],
            "pref": [5.0, 0.5, 0.01],
            "edges_per_step": 3,
            "directed": False,
            "seed": 9_993_012,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "ecount_min": 117,
            "ecount_max": 117,
            "no_self_loops": True,
            "max_type": 2,
        },
    },
    {
        "case": "cited_type_py_zero_pref_fallback_n12_eps1",
        "origin": "constructed (Graph.Cited_Type with pref=[0.0]): sum=0 "
        "fallback path ⇒ every citation is a self-loop on the step vertex",
        "algo": "cited_type_game",
        "params": {
            "nodes": 12,
            "types": [0 for _ in range(12)],
            "pref": [0.0],
            "edges_per_step": 1,
            "directed": True,
            "seed": 9_993_013,
        },
        "expected": {
            "vcount": 12,
            "directed": True,
            "ecount_min": 11,  # (12-1)*1 = 11
            "ecount_max": 11,
            "all_self_loops": True,
            "max_type": 0,
        },
    },
]

# ALGO-GN-029: citing_cited_type_game. Mirrors
# ig.Graph.Citing_Cited_Type(types, pref, edges_per_step, directed, ...) —
# Cython wrapper on `igraph_citing_cited_type_game`. Like cited_type but
# the citing vertex's category also influences the choice: weight is
# pref[type[citing]][type[cited]] (one psumtree per citing type). RNG
# state is not portable; structural invariants only. Multi-edges allowed
# when eps≥2; NEVER self-loops (uniform fallback samples [0, i)).
CITING_CITED_TYPE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "citing_cited_py_uniform_pref_n25_2types_eps2_directed",
        "origin": "tests/test_games.py-style — Graph.Citing_Cited_Type("
        "types=[0,1,...], pref=2x2 ones, edges_per_step=2, directed): "
        "uniform pref ⇒ exactly (n-1)*eps = 48 edges, no self-loops",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 25,
            "types": [v % 2 for v in range(25)],
            "pref": [[1.0, 1.0], [1.0, 1.0]],
            "edges_per_step": 2,
            "directed": True,
            "seed": 9_994_011,
        },
        "expected": {
            "vcount": 25,
            "directed": True,
            "ecount_min": 48,
            "ecount_max": 48,
            "no_self_loops": True,
            "max_type": 1,
        },
    },
    {
        "case": "citing_cited_py_disassortative_pref_n40_3types_eps3_undirected",
        "origin": "constructed (Graph.Citing_Cited_Type): three types with "
        "disassortative pref (high off-diagonal); positive pref ⇒ no "
        "self-loops, exactly (n-1)*eps = 117 edges",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 40,
            "types": [v % 3 for v in range(40)],
            "pref": [
                [0.1, 5.0, 5.0],
                [5.0, 0.1, 5.0],
                [5.0, 5.0, 0.1],
            ],
            "edges_per_step": 3,
            "directed": False,
            "seed": 9_994_012,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "ecount_min": 117,
            "ecount_max": 117,
            "no_self_loops": True,
            "max_type": 2,
        },
    },
    {
        "case": "citing_cited_py_row_zero_fallback_n15_2types_eps1_directed",
        "origin": "constructed (Graph.Citing_Cited_Type with row-zero pref): "
        "citing type 0 has all-zero weights ⇒ uniform fallback fires only "
        "for those steps; citing type 1 samples structurally. No self-loops.",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 15,
            "types": [v % 2 for v in range(15)],
            "pref": [[0.0, 0.0], [1.0, 1.0]],
            "edges_per_step": 1,
            "directed": True,
            "seed": 9_994_013,
        },
        "expected": {
            "vcount": 15,
            "directed": True,
            "ecount_min": 14,  # (15-1)*1 = 14
            "ecount_max": 14,
            "no_self_loops": True,
            "max_type": 1,
        },
    },
]

# ALGO-GN-018: lastcit_game. Mirrors ig.Graph.Lastcit / sample_last_cit
# (Cython wrapper on `igraph_lastcit_game`). Each new vertex emits
# `edges_per_node` outgoing citations; cited vertices' weights decay
# with the time since their last citation, binned into `agebins`
# buckets. The psumtree implementation gives O(log n) update + search.
# Never self-loops by construction; may produce multi-edges when
# edges_per_node ≥ 2.
LASTCIT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lastcit_py_uniform_n20_2bins_eps2",
        "origin": "constructed (mirrors igraph.Graph.Lastcit(n=20, "
        "edges_per_node=2, agebins=2, preference=[1,1,1], directed)) — "
        "uniform preference baseline; ecount = (n-1)*eps = 38",
        "algo": "lastcit_game",
        "params": {
            "nodes": 20,
            "edges_per_node": 2,
            "agebins": 2,
            "preference": [1.0, 1.0, 1.0],
            "directed": True,
            "seed": 9_994_001,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "ecount_min": 38,
            "ecount_max": 38,
            "no_self_loops": True,
        },
    },
    {
        "case": "lastcit_py_high_recent_n50_3bins_eps2",
        "origin": "constructed (mirrors igraph.Graph.Lastcit(n=50, "
        "edges_per_node=2, agebins=3, preference=[100,5,1,0.1], directed)) — "
        "strong recency preference; psumtree concentrates citations on the "
        "most recently cited cohort",
        "algo": "lastcit_game",
        "params": {
            "nodes": 50,
            "edges_per_node": 2,
            "agebins": 3,
            "preference": [100.0, 5.0, 1.0, 0.1],
            "directed": True,
            "seed": 9_994_002,
        },
        "expected": {
            "vcount": 50,
            "directed": True,
            "ecount_min": 98,
            "ecount_max": 98,
            "no_self_loops": True,
        },
    },
    {
        "case": "lastcit_py_single_agebin_n35_eps4_undirected",
        "origin": "constructed (mirrors igraph.Graph.Lastcit(n=35, "
        "edges_per_node=4, agebins=1, preference=[2,1], undirected)) — "
        "degenerate one-bin case: every cited vertex keeps weight 2 "
        "forever (no age sweep ever fires)",
        "algo": "lastcit_game",
        "params": {
            "nodes": 35,
            "edges_per_node": 4,
            "agebins": 1,
            "preference": [2.0, 1.0],
            "directed": False,
            "seed": 9_994_003,
        },
        "expected": {
            "vcount": 35,
            "directed": False,
            "ecount_min": 136,
            "ecount_max": 136,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-019: recent_degree_game. Mirrors ig.Graph.Recent_Degree
# (Cython wrapper on `igraph_recent_degree_game`). Each step draws m
# citations weighted by `pow(recent_in_degree, power) + zero_appeal`;
# edges added at step `i - time_window` are expired from the BIT-tree.
# Never self-loops by construction.
RECENT_DEGREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "recent_degree_py_pow1_window7_m2_directed",
        "origin": "constructed (mirrors ig.Graph.Recent_Degree(n=40, "
        "power=1.0, window=7, m=2, outpref=False, zero_appeal=1.0, "
        "directed)) — linear preferential attachment with a 7-step "
        "memory window; ecount = (40-1)*2 = 78",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 40,
            "power": 1.0,
            "time_window": 7,
            "m": 2,
            "outpref": False,
            "zero_appeal": 1.0,
            "directed": True,
            "seed": 9_996_001,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "ecount_min": 78,
            "ecount_max": 78,
            "no_self_loops": True,
        },
    },
    {
        "case": "recent_degree_py_high_power_short_window_m1",
        "origin": "constructed (mirrors ig.Graph.Recent_Degree(n=50, "
        "power=3.0, window=3, m=1, outpref=False, zero_appeal=0.1, "
        "directed)) — super-linear power with very short window; "
        "richest-recent-vertex wins almost every draw",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 50,
            "power": 3.0,
            "time_window": 3,
            "m": 1,
            "outpref": False,
            "zero_appeal": 0.1,
            "directed": True,
            "seed": 9_996_002,
        },
        "expected": {
            "vcount": 50,
            "directed": True,
            "ecount_min": 49,
            "ecount_max": 49,
            "no_self_loops": True,
        },
    },
    {
        "case": "recent_degree_py_time_window_zero_uniform_m3_undirected",
        "origin": "constructed (mirrors ig.Graph.Recent_Degree(n=30, "
        "power=1.5, window=0, m=3, outpref=False, zero_appeal=1.0, "
        "undirected)) — time_window=0 means everything expires "
        "immediately, so the BIT-tree only ever holds zero_appeal "
        "weights ⇒ uniform draws over existing vertices",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 30,
            "power": 1.5,
            "time_window": 0,
            "m": 3,
            "outpref": False,
            "zero_appeal": 1.0,
            "directed": False,
            "seed": 9_996_003,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "ecount_min": 87,  # (30-1)*3 = 87
            "ecount_max": 87,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-020: barabasi_game_psumtree / barabasi_game_psumtree_multiple.
# Mirrors ig.Graph.Barabasi(..., implementation="psumtree") and
# implementation="psumtree_multiple" (Cython wrapper on
# `igraph_barabasi_game`). The SIMPLE variant prevents within-step
# multi-edges via per-draw weight zeroing; the MULTIPLE variant snapshots
# the BIT sum once per step and uses the `m >= i` early-cite branch.
# Never self-loops by construction.
BARABASI_PSUMTREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_psumtree_py_classic_directed_m2",
        "origin": "constructed (mirrors ig.Graph.Barabasi(n=40, m=2, "
        "power=1.0, outpref=False, A=1.0, directed=True, "
        "implementation='psumtree')) — classical linear BA kernel",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 40,
            "power": 1.0,
            "m": 2,
            "outpref": False,
            "a": 1.0,
            "directed": True,
            "variant": "psumtree",
            "seed": 9_999_001,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "ecount_min": 78,
            "ecount_max": 78,
            "no_self_loops": True,
        },
    },
    {
        "case": "barabasi_psumtree_py_multiple_pow15_directed_m3",
        "origin": "constructed (mirrors ig.Graph.Barabasi(n=30, m=3, "
        "power=1.5, outpref=False, A=1.0, directed=True, "
        "implementation='psumtree_multiple')) — saturation triangle "
        "deducts 3 edges from the naive 87 total",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 30,
            "power": 1.5,
            "m": 3,
            "outpref": False,
            "a": 1.0,
            "directed": True,
            "variant": "psumtree_multiple",
            "seed": 9_999_002,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "ecount_min": 84,
            "ecount_max": 84,
            "no_self_loops": True,
        },
    },
    {
        "case": "barabasi_psumtree_py_undirected_outpref_m2",
        "origin": "constructed (mirrors ig.Graph.Barabasi(n=35, m=2, "
        "power=1.0, outpref=True, A=0.5, directed=False, "
        "implementation='psumtree')) — undirected forces outpref=True",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 35,
            "power": 1.0,
            "m": 2,
            "outpref": True,
            "a": 0.5,
            "directed": False,
            "variant": "psumtree",
            "seed": 9_999_003,
        },
        "expected": {
            "vcount": 35,
            "directed": False,
            "ecount_min": 68,
            "ecount_max": 68,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-021: barabasi_aging_game. The Python binding does not expose a
# direct `Barabasi_Aging` constructor (you can call it via
# `ig.Graph(...)` only after wiring up the kwargs manually), so the
# fixtures pin the structural invariants the C kernel guarantees:
# without `outseq`, ecount = (nodes - 1) * m exactly, no self-loops,
# and the directed flag and vcount are obvious. RNG state is not
# portable; expected ecount is exact (one edge per attempted draw).
BARABASI_AGING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_aging_py_classic_no_aging_directed_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=40, "
        "m=2, outpref=False, pa_exp=1.0, aging_exp=0.0, aging_bins=10, "
        "zero_deg_appeal=1.0, zero_age_appeal=1.0, deg_coef=1.0, "
        "age_coef=1.0, directed=True)) — aging_exp=0 degenerates to a "
        "constant age term, recovering classical BA up to scale",
        "algo": "barabasi_aging_game",
        "params": {
            "nodes": 40,
            "m": 2,
            "outpref": False,
            "pa_exp": 1.0,
            "aging_exp": 0.0,
            "aging_bins": 10,
            "zero_deg_appeal": 1.0,
            "zero_age_appeal": 1.0,
            "deg_coef": 1.0,
            "age_coef": 1.0,
            "directed": True,
            "seed": 9_999_101,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "ecount_min": 78,
            "ecount_max": 78,
            "no_self_loops": True,
        },
    },
    {
        "case": "barabasi_aging_py_strong_aging_directed_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=40, "
        "m=2, outpref=False, pa_exp=1.0, aging_exp=-1.0, aging_bins=10, "
        "zero_deg_appeal=1.0, zero_age_appeal=1.0, deg_coef=1.0, "
        "age_coef=1.0, directed=True)) — aging_exp=-1 favours fresh "
        "vertices",
        "algo": "barabasi_aging_game",
        "params": {
            "nodes": 40,
            "m": 2,
            "outpref": False,
            "pa_exp": 1.0,
            "aging_exp": -1.0,
            "aging_bins": 10,
            "zero_deg_appeal": 1.0,
            "zero_age_appeal": 1.0,
            "deg_coef": 1.0,
            "age_coef": 1.0,
            "directed": True,
            "seed": 9_999_102,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "ecount_min": 78,
            "ecount_max": 78,
            "no_self_loops": True,
        },
    },
    {
        "case": "barabasi_aging_py_outpref_undirected_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=35, "
        "m=2, outpref=True, pa_exp=1.0, aging_exp=-0.5, aging_bins=8, "
        "zero_deg_appeal=0.5, zero_age_appeal=1.0, deg_coef=1.0, "
        "age_coef=1.0, directed=False)) — undirected + outpref feeds "
        "the new vertex's own degree back into its weight",
        "algo": "barabasi_aging_game",
        "params": {
            "nodes": 35,
            "m": 2,
            "outpref": True,
            "pa_exp": 1.0,
            "aging_exp": -0.5,
            "aging_bins": 8,
            "zero_deg_appeal": 0.5,
            "zero_age_appeal": 1.0,
            "deg_coef": 1.0,
            "age_coef": 1.0,
            "directed": False,
            "seed": 9_999_103,
        },
        "expected": {
            "vcount": 35,
            "directed": False,
            "ecount_min": 68,
            "ecount_max": 68,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-022: dot_product_game. The Python binding does not expose a
# direct `ig.Graph.DotProduct(...)` factory; the C kernel
# `igraph_dot_product_game` is reached via the lower-level
# `_igraph._dot_product_game`. We mirror the *kernel* semantics here so
# our SplitMix64 backend produces structurally identical graphs (vcount,
# directed flag, ecount band, simple-by-construction). RNG state is not
# portable, so deterministic-ecount fixtures use latent vectors that
# clamp every dot-product to {0, 1} (always-edge or never-edge regimes),
# making ecount exact under any RNG. The third case exercises both
# warning regimes (negative + over-one).
DOT_PRODUCT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dot_product_py_all_ones_complete_n8_undirected",
        "origin": "constructed (mirrors igraph_dot_product_game with "
        "vecs[i] = [1.0] for i ∈ [0, 8), directed=false) — all dots = "
        "1.0; with strict `gen_unit() < prob` (gen_unit ∈ [0, 1)) every "
        "pair fires; ecount = 8·7/2 = 28 exact",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.0]] * 8,
            "directed": False,
            "seed": 10_002_101,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "ecount_min": 28,
            "ecount_max": 28,
            "no_self_loops": True,
        },
    },
    {
        "case": "dot_product_py_orthogonal_groups_n8_undirected",
        "origin": "constructed (mirrors igraph_dot_product_game with "
        "vecs = [[1,0]]*4 ++ [[0,1]]*4, directed=false) — same-group "
        "dot = 1 always edge, cross-group dot = 0 never edge; ecount = "
        "2·C(4,2) = 12 exact",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.0, 0.0]] * 4 + [[0.0, 1.0]] * 4,
            "directed": False,
            "seed": 10_002_102,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "ecount_min": 12,
            "ecount_max": 12,
            "no_self_loops": True,
        },
    },
    {
        "case": "dot_product_py_mixed_clamp_n10_directed",
        "origin": "constructed (mirrors igraph_dot_product_game with "
        "vecs = [[1.5]]*5 ++ [[-0.5]]*5, directed=true) — same-(+) dot "
        "= 2.25 always edge (no RNG draw, 5·4 = 20); same-(−) dot = "
        "0.25 Bernoulli (5·4 attempts → 0..20); cross dot = -0.75 always "
        "skip; ecount ∈ [20, 40]; exercises both clamp warnings",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.5]] * 5 + [[-0.5]] * 5,
            "directed": True,
            "seed": 10_002_103,
        },
        "expected": {
            "vcount": 10,
            "directed": True,
            "ecount_min": 20,
            "ecount_max": 40,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-023: correlated_game + correlated_pair_game. The python-igraph
# binding does not expose direct factories for these games (no
# `ig.Graph.Correlated(...)` or similar); the C kernels are reached via
# the low-level `_igraph._correlated_game` / `_igraph._correlated_pair_game`
# entry points. We mirror the *kernel* semantics here so our SplitMix64
# backend produces structurally identical graphs. RNG state is not
# portable to python-igraph's Mersenne Twister; structural-only fixtures
# pin corr=1.0 cases (exact copy of old graph ⇒ exact ecount) and use 6σ
# Binomial bands on the pair-game ecounts.
CORRELATED_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "correlated_py_corr1_path_n4_exact_copy",
        "origin": "constructed (mirrors `_igraph._correlated_game` with "
        "old = ig.Graph(n=4, edges=[(0,1),(1,2),(2,3)], directed=False), "
        "corr=1.0, p=0.5, no permutation) — corr=1 yields p_del=0 and "
        "p_add=0, so the new graph is exactly the old; ecount = 3 exact",
        "algo": "correlated_game",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3)],
            directed=False,
        ),
        "params": {
            "corr": 1.0,
            "p": 0.5,
            "permutation": None,
            "seed": 11_022_301,
        },
        "expected": {
            "vcount": 4,
            "directed": False,
            "ecount_min": 3,
            "ecount_max": 3,
            "no_self_loops": True,
            "is_simple": True,
        },
    },
    {
        "case": "correlated_py_corr1_cycle_n5_permutation_reverse",
        "origin": "constructed (mirrors `_igraph._correlated_game` with "
        "old = C5 cycle, corr=1.0, p=0.5, permutation=(4,3,2,1,0)) — "
        "permutation only relabels vertices, ecount = 5 exact",
        "algo": "correlated_game",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)],
            directed=False,
        ),
        "params": {
            "corr": 1.0,
            "p": 0.5,
            "permutation": [4, 3, 2, 1, 0],
            "seed": 11_022_302,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "ecount_min": 5,
            "ecount_max": 5,
            "no_self_loops": True,
            "is_simple": True,
        },
    },
]

CORRELATED_PAIR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "correlated_pair_py_n30_corr5_p2_undirected",
        "origin": "constructed (mirrors `_igraph._correlated_pair_game` "
        "with n=30, corr=0.5, p=0.2, directed=false) — both graphs are "
        "ER-marginal: mean ecount = C(30,2)·0.2 = 87, σ ≈ 8.34, "
        "conservative band [40, 140]",
        "algo": "correlated_pair_game",
        "params": {
            "n": 30,
            "corr": 0.5,
            "p": 0.2,
            "directed": False,
            "permutation": None,
            "seed": 11_022_311,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "ecount_min": 40,
            "ecount_max": 140,
            "no_self_loops": True,
            "is_simple": True,
        },
    },
    {
        "case": "correlated_pair_py_n20_corr8_p25_directed",
        "origin": "constructed (mirrors `_igraph._correlated_pair_game` "
        "with n=20, corr=0.8, p=0.25, directed=true) — both graphs are "
        "ER-marginal: mean ecount = 20·19·0.25 = 95, σ ≈ 8.44, "
        "conservative band [45, 150]",
        "algo": "correlated_pair_game",
        "params": {
            "n": 20,
            "corr": 0.8,
            "p": 0.25,
            "directed": True,
            "permutation": None,
            "seed": 11_022_312,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "ecount_min": 45,
            "ecount_max": 150,
            "no_self_loops": True,
            "is_simple": True,
        },
    },
]

# ALGO-GN-024: degree_sequence_game (CONFIGURATION). python-igraph exposes
# this as `ig.Graph.Degree_Sequence(out, in_=None, method="configuration")`.
# Like the C-level fixtures, configuration is degree-preserving by
# construction, so the expected outcome pins vcount, ecount and the full
# degree sequence — no bands needed.
DEGREE_SEQUENCE_CONFIG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_config_py_undirected_n8_uniform_d3",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[3]*8, method='configuration')`): 8 vertices, all degree 3, "
        "Σd=24 (even). Multigraph (may have loops/multi-edges).",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "seed": 9_240_001,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "ecount": 12,
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
        },
    },
    {
        "case": "degseq_config_py_directed_n6_mixed",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "out=[2,1,3,0,1,2], in_=[1,2,1,2,1,2], method='configuration')`): "
        "directed multigraph; Σout=Σin=9.",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [2, 1, 3, 0, 1, 2],
            "in_degrees": [1, 2, 1, 2, 1, 2],
            "seed": 9_240_002,
        },
        "expected": {
            "vcount": 6,
            "directed": True,
            "ecount": 9,
            "out_degrees": [2, 1, 3, 0, 1, 2],
            "in_degrees": [1, 2, 1, 2, 1, 2],
        },
    },
    {
        "case": "degseq_config_py_singleton",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence([0])` "
        "with method='configuration'): one isolated vertex, no edges.",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [0],
            "in_degrees": None,
            "seed": 9_240_003,
        },
        "expected": {
            "vcount": 1,
            "directed": False,
            "ecount": 0,
            "out_degrees": [0],
            "in_degrees": None,
        },
    },
]

# ALGO-GN-026: degree_sequence_game (FAST_HEUR_SIMPLE method).
# python-igraph exposes this as
# `ig.Graph.Degree_Sequence(out, in_=None, method="fast_heur_simple")`,
# returning a simple (no self-loops, no multi-edges) graph that exactly
# realises the supplied degree sequence. RNG state is not portable, so the
# fixtures pin only structural invariants — vcount, ecount, exact degrees,
# simplicity.
DEGREE_SEQUENCE_FAST_HEUR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_fastheur_py_undirected_n8_uniform_d3",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[3]*8, method='fast_heur_simple')`): 8 vertices all degree 3, "
        "Σd=24. FAST_HEUR_SIMPLE guarantees simple (loop/multi-edge free).",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "seed": 9_260_001,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "ecount": 12,
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_fastheur_py_undirected_n10_skewed",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[5,4,4,3,3,3,2,2,2,2], method='fast_heur_simple')`): 10 "
        "vertices, mixed skewed degrees, Σd=30.",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [5, 4, 4, 3, 3, 3, 2, 2, 2, 2],
            "in_degrees": None,
            "seed": 9_260_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 15,
            "out_degrees": [5, 4, 4, 3, 3, 3, 2, 2, 2, 2],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_fastheur_py_directed_n6_skewed",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "out=[3,2,2,1,1,1], in_=[2,2,2,1,2,1], method='fast_heur_simple')`)"
        ": directed simple graph, Σout=Σin=10.",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [3, 2, 2, 1, 1, 1],
            "in_degrees": [2, 2, 2, 1, 2, 1],
            "seed": 9_260_003,
        },
        "expected": {
            "vcount": 6,
            "directed": True,
            "ecount": 10,
            "out_degrees": [3, 2, 2, 1, 1, 1],
            "in_degrees": [2, 2, 2, 1, 2, 1],
            "is_simple": True,
        },
    },
]

# ALGO-GN-027: degree_sequence_game (CONFIGURATION_SIMPLE method).
# python-igraph exposes this as `ig.Graph.Degree_Sequence(out, in_,
# method="configuration_simple")`. The CONFIGURATION_SIMPLE method uses
# stub-matching with two-swap-per-edge incremental Fisher-Yates and
# restarts on every self-loop or multi-edge encountered, returning a
# uniformly-distributed simple graph with the exact degree sequence.
# RNG state is not portable, so fixtures pin only structural invariants:
# vcount, ecount=Σd/2 (undirected) or Σd (directed), exact degrees,
# simplicity. Density is kept moderate because expected restart count
# grows as exp(O((Σd/n)²)) for this sampler.
DEGREE_SEQUENCE_CONFIG_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_configsimple_py_undirected_n8_uniform_d3",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[3]*8, method='configuration_simple')`): 8 vertices all degree "
        "3, Σd=24. CONFIGURATION_SIMPLE guarantees a uniformly-sampled "
        "simple graph realising the sequence.",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "seed": 9_270_001,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "ecount": 12,
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_configsimple_py_undirected_n10_skewed",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[4,3,3,3,2,2,2,2,2,1], method='configuration_simple')`): 10 "
        "vertices, moderately skewed sequence, Σd=24 (moderate density "
        "to keep rejection-sampling tractable).",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [4, 3, 3, 3, 2, 2, 2, 2, 2, 1],
            "in_degrees": None,
            "seed": 9_270_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 12,
            "out_degrees": [4, 3, 3, 3, 2, 2, 2, 2, 2, 1],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_configsimple_py_directed_n6_skewed",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "out=[2,2,2,1,1,1], in_=[2,1,2,1,2,1], "
        "method='configuration_simple')`): directed simple graph, "
        "Σout=Σin=9.",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [2, 2, 2, 1, 1, 1],
            "in_degrees": [2, 1, 2, 1, 2, 1],
            "seed": 9_270_003,
        },
        "expected": {
            "vcount": 6,
            "directed": True,
            "ecount": 9,
            "out_degrees": [2, 2, 2, 1, 1, 1],
            "in_degrees": [2, 1, 2, 1, 2, 1],
            "is_simple": True,
        },
    },
]

# ALGO-GN-028: degree_sequence_game (EDGE_SWITCHING_SIMPLE method).
# python-igraph exposes this as `ig.Graph.Degree_Sequence(out, in_,
# method="edge_switching_simple")`. Two-phase: deterministic
# Havel-Hakimi / Kleitman-Wang INDEX seed, then 10·|E| edge-switching
# MCMC trials. Cost is linear in |E| regardless of density, so this
# sampler handles dense / skewed sequences that exceed
# CONFIGURATION_SIMPLE's restart budget. Pins structural invariants
# only: vcount, ecount = Σd/2 (undirected) or Σout (directed), exact
# (out/in-)degree match, is_simple.
DEGREE_SEQUENCE_EDGE_SWITCHING_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_edge_switching_py_undirected_n10_skewed_dense",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[5,4,4,3,3,3,2,2,2,2], method='edge_switching_simple')`): "
        "n=10, Σd=30, density Σd/n=3 — a regime where "
        "CONFIGURATION_SIMPLE rejects often but EDGE_SWITCHING_SIMPLE "
        "remains linear in |E|.",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [5, 4, 4, 3, 3, 3, 2, 2, 2, 2],
            "in_degrees": None,
            "seed": 9_280_001,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 15,
            "out_degrees": [5, 4, 4, 3, 3, 3, 2, 2, 2, 2],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_edge_switching_py_undirected_n12_4regular",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[4]*12, method='edge_switching_simple')`): 4-regular on 12 "
        "vertices, Σd=48, density Σd/n=4 — dense regime tractable "
        "for EDGE_SWITCHING_SIMPLE.",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            "in_degrees": None,
            "seed": 9_280_002,
        },
        "expected": {
            "vcount": 12,
            "directed": False,
            "ecount": 24,
            "out_degrees": [4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_edge_switching_py_directed_n8_balanced_d2",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[2]*8, in_=[2]*8, method='edge_switching_simple')`): "
        "directed balanced (out=in=2 everywhere) on n=8, Σ=16.",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [2, 2, 2, 2, 2, 2, 2, 2],
            "in_degrees": [2, 2, 2, 2, 2, 2, 2, 2],
            "seed": 9_280_003,
        },
        "expected": {
            "vcount": 8,
            "directed": True,
            "ecount": 16,
            "out_degrees": [2, 2, 2, 2, 2, 2, 2, 2],
            "in_degrees": [2, 2, 2, 2, 2, 2, 2, 2],
            "is_simple": True,
        },
    },
]

# ALGO-GN-025: degree_sequence_game (VL method). python-igraph exposes
# this as `ig.Graph.Degree_Sequence(out, method="vl")` (undirected only).
# The VL method samples a connected, simple undirected graph that exactly
# realises the degree sequence — invariants pinned: vcount, ecount=Σd/2,
# exact degree match, simplicity, weak connectivity. RNG state is not
# shared with Rust's SplitMix64, so the manifest does not require edge-
# for-edge agreement.
DEGREE_SEQUENCE_VL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_vl_py_undirected_n8_uniform_d3",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[3]*8, method='vl')`): 8 vertices all degree 3, Σd=24. "
        "VL guarantees simple and connected.",
        "algo": "degree_sequence_game_vl",
        "params": {
            "degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "seed": 9_250_001,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "ecount": 12,
            "degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "is_simple": True,
            "is_connected": True,
        },
    },
    {
        "case": "degseq_vl_py_undirected_n10_skewed",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[5,4,4,3,3,3,2,2,2,2], method='vl')`): 10 vertices, mixed "
        "skewed degrees, Σd=30.",
        "algo": "degree_sequence_game_vl",
        "params": {
            "degrees": [5, 4, 4, 3, 3, 3, 2, 2, 2, 2],
            "seed": 9_250_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 15,
            "degrees": [5, 4, 4, 3, 3, 3, 2, 2, 2, 2],
            "is_simple": True,
            "is_connected": True,
        },
    },
    {
        "case": "degseq_vl_py_singleton_zero",
        "origin": "constructed (mirrors `ig.Graph.Degree_Sequence("
        "[0], method='vl')`): one isolated vertex, no edges.",
        "algo": "degree_sequence_game_vl",
        "params": {
            "degrees": [0],
            "seed": 9_250_003,
        },
        "expected": {
            "vcount": 1,
            "directed": False,
            "ecount": 0,
            "degrees": [0],
            "is_simple": True,
            "is_connected": True,
        },
    },
]

# ALGO-GN-007: simple_interconnected_islands_game. Mirrors
# `ig.Graph.SBM`-like factory `ig.Graph.SimpleInterconnectedIslands(
# islands_n, islands_size, islands_pin, n_inter)` (Cython wrapper on
# `igraph_simple_interconnected_islands_game`). RNG state is not
# portable; we encode structural invariants — vcount, directed=False,
# is_simple (no loops + no parallels), and an ecount band built from
# expected_intra = islands_n * C(size, 2) * pin and exact
# expected_inter = C(islands_n, 2) * n_inter.
ISLANDS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "islands_py_3islands_size15_pin04_inter1",
        "origin": "constructed (mirrors ig.Graph.SimpleInterconnectedIslands("
        "islands_n=3, islands_size=15, islands_pin=0.4, n_inter=1)): "
        "three medium islands with one cross-bridge each",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 3,
            "islands_size": 15,
            "islands_pin": 0.4,
            "n_inter": 1,
            "seed": 7_770_101,
        },
        "expected": {
            "vcount": 45,
            "directed": False,
            "is_simple": True,
            # E[intra] = 3 * 15*14/2 * 0.4 = 126; exact_inter = 3*1 = 3.
            # Band [0.6*126 + 3, 1.4*126 + 3] = [78, 179].
            "ecount_min": 78,
            "ecount_max": 179,
        },
    },
    {
        "case": "islands_py_pin0_pure_bipartite",
        "origin": "constructed (mirrors ig.Graph.SimpleInterconnectedIslands("
        "islands_n=4, islands_size=8, islands_pin=0.0, n_inter=2)): "
        "no intra edges → exact C(4,2)·2 = 12 inter-island edges",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 4,
            "islands_size": 8,
            "islands_pin": 0.0,
            "n_inter": 2,
            "seed": 7_770_102,
        },
        "expected": {
            "vcount": 32,
            "directed": False,
            "is_simple": True,
            "ecount_min": 12,
            "ecount_max": 12,
        },
    },
    {
        "case": "islands_py_saturated_bipartite",
        "origin": "constructed (mirrors ig.Graph.SimpleInterconnectedIslands("
        "islands_n=2, islands_size=4, islands_pin=0.5, n_inter=16)): "
        "n_inter = size² saturates the bipartite slice",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 2,
            "islands_size": 4,
            "islands_pin": 0.5,
            "n_inter": 16,
            "seed": 7_770_103,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "is_simple": True,
            # E[intra] = 2 * 4*3/2 * 0.5 = 6; exact_inter = 1*16 = 16.
            # Band [0.6*6 + 16, 1.4*6 + 16] = [19, 25].
            "ecount_min": 19,
            "ecount_max": 25,
        },
    },
]

K_REGULAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "k_regular_py_undirected_simple_n12_k4",
        "origin": "constructed (mirrors ig.Graph.K_Regular(n=12, k=4, "
        "directed=False, multiple=False)): every vertex has degree 4",
        "algo": "k_regular_game",
        "params": {
            "n": 12,
            "k": 4,
            "directed": False,
            "multiple": False,
            "seed": 7_770_201,
        },
        "expected": {
            "vcount": 12,
            "directed": False,
            "is_simple": True,
            "ecount_min": 24,
            "ecount_max": 24,
            "every_degree": 4,
        },
    },
    {
        "case": "k_regular_py_directed_simple_n6_k2",
        "origin": "constructed (mirrors ig.Graph.K_Regular(n=6, k=2, "
        "directed=True, multiple=False)): every vertex has "
        "out-degree = in-degree = 2",
        "algo": "k_regular_game",
        "params": {
            "n": 6,
            "k": 2,
            "directed": True,
            "multiple": False,
            "seed": 7_770_202,
        },
        "expected": {
            "vcount": 6,
            "directed": True,
            "is_simple": True,
            "ecount_min": 12,
            "ecount_max": 12,
            "every_out_degree": 2,
            "every_in_degree": 2,
        },
    },
    {
        "case": "k_regular_py_k_zero_isolated",
        "origin": "constructed (mirrors ig.Graph.K_Regular(n=7, k=0)): "
        "edge-less 7-vertex graph with every vertex isolated",
        "algo": "k_regular_game",
        "params": {
            "n": 7,
            "k": 0,
            "directed": False,
            "multiple": False,
            "seed": 7_770_203,
        },
        "expected": {
            "vcount": 7,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
            "every_degree": 0,
        },
    },
]

WATTS_STROGATZ_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "watts_py_ring_lattice_p0_size12_nei2",
        "origin": "constructed (mirrors ig.Graph.Watts_Strogatz(dim=1, "
        "size=12, nei=2, p=0)): pure ring lattice, every vertex has "
        "degree 4, edges = size * nei = 24",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 12,
            "nei": 2,
            "p": 0.0,
            "loops": False,
            "multiple": False,
            "seed": 9_100_001,
        },
        "expected": {
            "vcount": 12,
            "directed": False,
            "is_simple": True,
            "ecount_min": 24,
            "ecount_max": 24,
            "every_degree": 4,
        },
    },
    {
        "case": "watts_py_small_world_p_low_size50_nei3",
        "origin": "constructed (mirrors ig.Graph.Watts_Strogatz(dim=1, "
        "size=50, nei=3, p=0.1)): small-world regime — most of the ring "
        "preserved with a few long-range rewires, still simple",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 50,
            "nei": 3,
            "p": 0.1,
            "loops": False,
            "multiple": False,
            "seed": 9_100_002,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            "is_simple": True,
            "ecount_min": 150,
            "ecount_max": 150,
        },
    },
    {
        "case": "watts_py_multigraph_loops_size10_nei1",
        "origin": "constructed (mirrors ig.Graph.Watts_Strogatz(dim=1, "
        "size=10, nei=1, p=0.8, loops=true, multiple=true)): permissive "
        "regime — self-loops and parallels allowed; edge count preserved",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 10,
            "nei": 1,
            "p": 0.8,
            "loops": True,
            "multiple": True,
            "seed": 9_100_003,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": False,  # multi + loops allowed
            "ecount_min": 10,
            "ecount_max": 10,
        },
    },
]

# ALGO-GN-010: sbm_game. Mirrors ig.Graph.SBM. RNG state is not
# portable across implementations, so each fixture pins parameter
# values and bands the structural invariants:
#   * vcount = sum(block_sizes) (exact);
#   * directed matches the flag;
#   * ecount lies in a generous band around the model mean;
#   * is_simple when loops=false and multiple=false;
#   * when the pref matrix is block-diagonal, every edge stays
#     on-diagonal (encoded via `diagonal_only_pref: true`).
SBM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "sbm_py_single_block_n25_p_quarter",
        "origin": "constructed (mirrors ig.Graph.SBM with n=25, "
        "pref_matrix=[[0.25]], block_sizes=[25], directed=false, "
        "loops=false): single-block reduces to undirected G(n,p)",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [[0.25]],
            "block_sizes": [25],
            "directed": False,
            "loops": False,
            "multiple": False,
            "seed": 10_100_001,
        },
        "expected": {
            "vcount": 25,
            "directed": False,
            "is_simple": True,
            "ecount_min": 40,
            "ecount_max": 140,
        },
    },
    {
        "case": "sbm_py_three_blocks_assortative",
        "origin": "constructed (mirrors ig.Graph.SBM with sizes=[12, 12, 12], "
        "in-block p=0.3, between-block p=0.04, undirected, no loops): "
        "three communities with weak inter-block coupling",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [[0.3, 0.04, 0.04], [0.04, 0.3, 0.04], [0.04, 0.04, 0.3]],
            "block_sizes": [12, 12, 12],
            "directed": False,
            "loops": False,
            "multiple": False,
            "seed": 10_100_002,
        },
        "expected": {
            "vcount": 36,
            "directed": False,
            "is_simple": True,
            "ecount_min": 40,
            "ecount_max": 150,
        },
    },
    {
        "case": "sbm_py_directed_two_blocks_asymmetric",
        "origin": "constructed (mirrors ig.Graph.SBM with sizes=[10, 10], "
        "asymmetric pref=[[0.2, 0.3], [0.05, 0.2]], directed=true, "
        "no loops): directed SBM, pref need not be symmetric",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [[0.2, 0.3], [0.05, 0.2]],
            "block_sizes": [10, 10],
            "directed": True,
            "loops": False,
            "multiple": False,
            "seed": 10_100_003,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "is_simple": True,
            "ecount_min": 30,
            "ecount_max": 130,
        },
    },
]

# ALGO-GN-011: hsbm_game. python-igraph does not expose HSBM under a
# dedicated factory (the C `igraph_hsbm_game` is unwrapped at the time
# of writing). Each fixture stays deterministic by pinning corner-case
# probability values (p=0 isolates macros; p=1 fully connects across
# macros) so the resulting ecount can be pinned exactly without needing
# the Python binding to roll its own RNG. Structural invariants:
#   * vcount = n;
#   * directed = false;
#   * ecount band — exact when p∈{0, 1}, otherwise a wide model band;
#   * is_simple = true (HSBM never produces loops or multi-edges).
HSBM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hsbm_py_two_macros_p0_full_intra",
        "origin": "constructed (mirrors what an ig.Graph.HSBM(n=12, m=6, "
        "rho=[0.5, 0.5], C=[[1.0, 1.0], [1.0, 1.0]], p=0) call would do): "
        "two macros, each = K_6 (intra micro-pair fully connected), "
        "no inter — exactly 2 × C(6,2) = 30 edges",
        "algo": "hsbm_game",
        "params": {
            "n": 12,
            "m": 6,
            "rho": [0.5, 0.5],
            "c": [[1.0, 1.0], [1.0, 1.0]],
            "p": 0.0,
            "seed": 11_010_001,
        },
        "expected": {
            "vcount": 12,
            "directed": False,
            "is_simple": True,
            "ecount_min": 30,
            "ecount_max": 30,
        },
    },
    {
        "case": "hsbm_py_two_macros_p1_no_intra",
        "origin": "constructed (mirrors what an ig.Graph.HSBM(n=12, m=6, "
        "rho=[0.5, 0.5], C=[[0.0, 0.0], [0.0, 0.0]], p=1) call would do): "
        "two macros with empty intra, full inter K_{6,6}=36 edges only",
        "algo": "hsbm_game",
        "params": {
            "n": 12,
            "m": 6,
            "rho": [0.5, 0.5],
            "c": [[0.0, 0.0], [0.0, 0.0]],
            "p": 1.0,
            "seed": 11_010_002,
        },
        "expected": {
            "vcount": 12,
            "directed": False,
            "is_simple": True,
            "ecount_min": 36,
            "ecount_max": 36,
        },
    },
    {
        "case": "hsbm_py_two_macros_mid_p_band",
        "origin": "constructed (mirrors ig.Graph.HSBM(n=20, m=10, "
        "rho=[0.3, 0.7], C=[[0.5, 0.2], [0.2, 0.5]], p=0.3)): mid-density "
        "fixture; ecount falls in a generous model band",
        "algo": "hsbm_game",
        "params": {
            "n": 20,
            "m": 10,
            "rho": [0.3, 0.7],
            "c": [[0.5, 0.2], [0.2, 0.5]],
            "p": 0.3,
            "seed": 11_010_003,
        },
        "expected": {
            "vcount": 20,
            "directed": False,
            "is_simple": True,
            "ecount_min": 30,
            "ecount_max": 140,
        },
    },
]

# ALGO-GN-011: hsbm_list_game. python-igraph does not expose this
# either, so fixtures pin corner-case p values for exact ecounts.
HSBM_LIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hsbm_list_py_two_unequal_macros_p0",
        "origin": "constructed (mirrors an ig.Graph.HSBMList(n=15, "
        "m_list=[5, 10], rho_list=[[1.0], [0.5, 0.5]], c_list=[[[1.0]], "
        "[[1.0, 1.0], [1.0, 1.0]]], p=0) call): macro 1 = K_5 = 10 edges, "
        "macro 2 = K_10 = 45 edges, no inter — 55 edges total",
        "algo": "hsbm_list_game",
        "params": {
            "n": 15,
            "m_list": [5, 10],
            "rho_list": [[1.0], [0.5, 0.5]],
            "c_list": [
                [[1.0]],
                [[1.0, 1.0], [1.0, 1.0]],
            ],
            "p": 0.0,
            "seed": 11_110_001,
        },
        "expected": {
            "vcount": 15,
            "directed": False,
            "is_simple": True,
            "ecount_min": 55,
            "ecount_max": 55,
        },
    },
    {
        "case": "hsbm_list_py_two_unequal_macros_p1",
        "origin": "constructed (mirrors the same HSBMList shape above "
        "but with p=1): 55 intra + K_{5,10}=50 inter = 105 edges",
        "algo": "hsbm_list_game",
        "params": {
            "n": 15,
            "m_list": [5, 10],
            "rho_list": [[1.0], [0.5, 0.5]],
            "c_list": [
                [[1.0]],
                [[1.0, 1.0], [1.0, 1.0]],
            ],
            "p": 1.0,
            "seed": 11_110_002,
        },
        "expected": {
            "vcount": 15,
            "directed": False,
            "is_simple": True,
            "ecount_min": 105,
            "ecount_max": 105,
        },
    },
    {
        "case": "hsbm_list_py_three_macros_p_band",
        "origin": "constructed (mirrors ig.Graph.HSBMList(n=30, "
        "m_list=[10, 10, 10], rho_list=[[0.5,0.5], [0.3,0.7], [1.0]], "
        "c_list=[block-anti, block-diag, scalar 0.5], p=0.2)): mid-density "
        "three-macro fixture; ecount lies in a generous model band",
        "algo": "hsbm_list_game",
        "params": {
            "n": 30,
            "m_list": [10, 10, 10],
            "rho_list": [[0.5, 0.5], [0.3, 0.7], [1.0]],
            "c_list": [
                [[0.0, 1.0], [1.0, 0.0]],
                [[0.5, 0.0], [0.0, 0.5]],
                [[0.5]],
            ],
            "p": 0.2,
            "seed": 11_110_003,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 60,
            "ecount_max": 260,
        },
    },
]

# ALGO-GN-012: chung_lu_game. python-igraph exposes Graph.Chung_Lu(out,
# in_=None, loops=True, variant="original") — see
# references/python-igraph/src/_igraph/graphobject.c lines 2200-2240.
# RNG is not portable across implementations, so fixtures pin vcount,
# directedness, and (when loops=False) is_simple, plus an exact ecount
# in the zero-weight degenerate cases. Variants exercised: original,
# maxent, nr. Both undirected (in_=None) and directed (in_=list) shapes
# are covered.
CHUNG_LU_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "chung_lu_py_zero_weights_loops_true_empty",
        "origin": "constructed (mirrors Graph.Chung_Lu([0]*6, loops=True, "
        "variant='original')): all-zero out → 0 edges.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            "in_weights": None,
            "loops": True,
            "variant": "original",
            "seed": 12_010_001,
        },
        "expected": {
            "vcount": 6,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "chung_lu_py_original_undirected_no_loops",
        "origin": "constructed (mirrors Graph.Chung_Lu([3, 3, 2, 2, 1, 1], "
        "loops=False, variant='original')): same weights as the R test "
        "in test-games.R:175 — original variant on n=6, low expected "
        "degrees → simple graph.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [3.0, 3.0, 2.0, 2.0, 1.0, 1.0],
            "in_weights": None,
            "loops": False,
            "variant": "original",
            "seed": 12_010_002,
        },
        "expected": {
            "vcount": 6,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 15,
        },
    },
    {
        "case": "chung_lu_py_maxent_undirected_no_loops",
        "origin": "constructed (mirrors Graph.Chung_Lu([3, 3, 2, 2, 1, 1], "
        "loops=False, variant='maxent')): same weights, maxent variant.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [3.0, 3.0, 2.0, 2.0, 1.0, 1.0],
            "in_weights": None,
            "loops": False,
            "variant": "maxent",
            "seed": 12_010_003,
        },
        "expected": {
            "vcount": 6,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 15,
        },
    },
    {
        "case": "chung_lu_py_nr_undirected_no_loops",
        "origin": "constructed (mirrors Graph.Chung_Lu([3, 3, 2, 2, 1, 1], "
        "loops=False, variant='nr')): same weights, NR variant.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [3.0, 3.0, 2.0, 2.0, 1.0, 1.0],
            "in_weights": None,
            "loops": False,
            "variant": "nr",
            "seed": 12_010_004,
        },
        "expected": {
            "vcount": 6,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 15,
        },
    },
    {
        "case": "chung_lu_py_directed_original_no_loops",
        "origin": "constructed (mirrors Graph.Chung_Lu([1, 3, 2, 1], "
        "in_=[2, 1, 2, 2], loops=False, variant='original')): mirrors "
        "the R doc-example call in games.R:3104; in/out sums both = 7.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 3.0, 2.0, 1.0],
            "in_weights": [2.0, 1.0, 2.0, 2.0],
            "loops": False,
            "variant": "original",
            "seed": 12_010_005,
        },
        "expected": {
            "vcount": 4,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 12,
        },
    },
    {
        "case": "chung_lu_py_directed_maxent_no_loops",
        "origin": "constructed (mirrors Graph.Chung_Lu([1, 3, 2, 1], "
        "in_=[2, 1, 2, 2], loops=False, variant='maxent')): mirrors the "
        "second R doc example with maxent variant.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 3.0, 2.0, 1.0],
            "in_weights": [2.0, 1.0, 2.0, 2.0],
            "loops": False,
            "variant": "maxent",
            "seed": 12_010_006,
        },
        "expected": {
            "vcount": 4,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 12,
        },
    },
    {
        "case": "chung_lu_py_large_n_original_band",
        "origin": "constructed (mirrors Graph.Chung_Lu(uniform 1.0 weights "
        "of length 30, loops=False, variant='original')): uniform weights "
        "give q = 1/30 for every off-diagonal pair; expected edges ≈ "
        "0.5*C(30,2) = 217.5; band is wide.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0] * 30,
            "in_weights": None,
            "loops": False,
            "variant": "original",
            "seed": 12_010_007,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 5,
            "ecount_max": 435,
        },
    },
    {
        "case": "chung_lu_py_vertex_count_single",
        "origin": "constructed (mirrors Graph.Chung_Lu([1.0], loops=True, "
        "variant='original')): single vertex; loops=True allows a self "
        "loop but never a parallel one.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0],
            "in_weights": None,
            "loops": True,
            "variant": "original",
            "seed": 12_010_008,
        },
        "expected": {
            "vcount": 1,
            "directed": False,
            "no_multi_edges": True,
            "ecount_min": 0,
            "ecount_max": 1,
        },
    },
    {
        "case": "chung_lu_py_directed_nr_no_loops",
        "origin": "constructed (mirrors Graph.Chung_Lu([1, 3, 2, 1], "
        "in_=[2, 1, 2, 2], loops=False, variant='nr')): exercises the "
        "NR variant under the directed shape.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 3.0, 2.0, 1.0],
            "in_weights": [2.0, 1.0, 2.0, 2.0],
            "loops": False,
            "variant": "nr",
            "seed": 12_010_009,
        },
        "expected": {
            "vcount": 4,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 12,
        },
    },
]


# ALGO-GN-013 (static_fitness_game). python-igraph exposes
# Graph.Static_Fitness(m, fitness_out, fitness_in=None, loops=False,
# multiple=False) — see references/python-igraph/src/_igraph/graphobject.c
# StaticFitness binding. Cases here mirror the binding's documented
# behaviour: empty graph, undirected/directed shapes, simple/loops/multi
# combinations. RNG state is not portable, so structural invariants only.
STATIC_FITNESS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "static_fitness_py_zero_edges_undirected",
        "origin": "constructed (mirrors Graph.Static_Fitness(0, "
        "[1,2,3,4,5])): m=0 yields five isolated vertices.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 0,
            "fitness_out": [1.0, 2.0, 3.0, 4.0, 5.0],
            "fitness_in": None,
            "loops": False,
            "multiple": False,
            "seed": 12_011_001,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "static_fitness_py_undirected_simple",
        "origin": "constructed (mirrors Graph.Static_Fitness(15, "
        "[1,2,3,4,5,6,7,8])): undirected simple, capacity C(8,2)=28 ≥ 15.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 15,
            "fitness_out": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            "fitness_in": None,
            "loops": False,
            "multiple": False,
            "seed": 12_011_002,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "is_simple": True,
            "ecount_min": 15,
            "ecount_max": 15,
        },
    },
    {
        "case": "static_fitness_py_undirected_multi_loops",
        "origin": "constructed (mirrors Graph.Static_Fitness(20, "
        "[2]*5, loops=True, multiple=True)): permissive — any pair "
        "including self-loops; ecount = m exactly.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 20,
            "fitness_out": [2.0, 2.0, 2.0, 2.0, 2.0],
            "fitness_in": None,
            "loops": True,
            "multiple": True,
            "seed": 12_011_003,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "ecount_min": 20,
            "ecount_max": 20,
        },
    },
    {
        "case": "static_fitness_py_directed_simple",
        "origin": "constructed (mirrors Graph.Static_Fitness(20, "
        "[1,2,3,4,5,6], [1,2,3,4,5,6])): directed simple — fitness_in "
        "list provided. Capacity n*(n-1) = 30 ≥ 20.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 20,
            "fitness_out": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "fitness_in": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "loops": False,
            "multiple": False,
            "seed": 12_011_004,
        },
        "expected": {
            "vcount": 6,
            "directed": True,
            "is_simple": True,
            "ecount_min": 20,
            "ecount_max": 20,
        },
    },
    {
        "case": "static_fitness_py_directed_loops_only",
        "origin": "constructed (mirrors Graph.Static_Fitness(15, "
        "[1,2,3,4,5,6], [6,5,4,3,2,1], loops=True)): directed, "
        "loops allowed but parallel edges forbidden.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 15,
            "fitness_out": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "fitness_in": [6.0, 5.0, 4.0, 3.0, 2.0, 1.0],
            "loops": True,
            "multiple": False,
            "seed": 12_011_005,
        },
        "expected": {
            "vcount": 6,
            "directed": True,
            "is_simple": False,
            "no_multi_edges": True,
            "ecount_min": 15,
            "ecount_max": 15,
        },
    },
]


# ALGO-GN-013 (static_power_law_game). python-igraph exposes
# Graph.Static_Power_Law(n, m, exponent_out, exponent_in=-1, loops=False,
# multiple=False, finite_size_correction=True). A negative exponent_in
# selects undirected. Cases mirror the canonical happy paths the C
# reference exercises, plus a parity case under FSC=False.
STATIC_POWER_LAW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "static_power_law_py_zero_edges_undirected",
        "origin": "constructed (mirrors Graph.Static_Power_Law(8, 0, 2.5)): "
        "m=0 yields isolated graph regardless of exponent.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 8,
            "no_of_edges": 0,
            "exponent_out": 2.5,
            "exponent_in": None,
            "loops": False,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_011_101,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "static_power_law_py_undirected_simple",
        "origin": "constructed (mirrors Graph.Static_Power_Law(50, 80, "
        "2.5, finite_size_correction=True)): undirected simple. "
        "Capacity C(50,2) = 1225 ≫ 80.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 50,
            "no_of_edges": 80,
            "exponent_out": 2.5,
            "exponent_in": None,
            "loops": False,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_011_102,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            "is_simple": True,
            "ecount_min": 80,
            "ecount_max": 80,
        },
    },
    {
        "case": "static_power_law_py_undirected_no_fsc",
        "origin": "constructed (mirrors Graph.Static_Power_Law(60, 100, "
        "3.0, finite_size_correction=False)): exponent above the FSC "
        "threshold — `α = -1/(γ-1) = -0.5`, no shift required.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 60,
            "no_of_edges": 100,
            "exponent_out": 3.0,
            "exponent_in": None,
            "loops": False,
            "multiple": False,
            "finite_size_correction": False,
            "seed": 12_011_103,
        },
        "expected": {
            "vcount": 60,
            "directed": False,
            "is_simple": True,
            "ecount_min": 100,
            "ecount_max": 100,
        },
    },
    {
        "case": "static_power_law_py_directed_loops_multi",
        "origin": "constructed (mirrors Graph.Static_Power_Law(40, 60, "
        "2.5, exponent_in=2.5, loops=True, multiple=True)): directed, "
        "fully permissive.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 40,
            "no_of_edges": 60,
            "exponent_out": 2.5,
            "exponent_in": 2.5,
            "loops": True,
            "multiple": True,
            "finite_size_correction": True,
            "seed": 12_011_104,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "ecount_min": 60,
            "ecount_max": 60,
        },
    },
    {
        "case": "static_power_law_py_undirected_multi_only",
        "origin": "constructed (mirrors Graph.Static_Power_Law(30, 80, "
        "2.5, multiple=True)): multi parallel edges allowed, no loops.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 30,
            "no_of_edges": 80,
            "exponent_out": 2.5,
            "exponent_in": None,
            "loops": False,
            "multiple": True,
            "finite_size_correction": True,
            "seed": 12_011_105,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "ecount_min": 80,
            "ecount_max": 80,
        },
    },
]


# ALGO-CN-001: ring (python-igraph factory `Graph.Ring(n, directed,
# mutual, circular)`). Construction is fully deterministic; expected
# edges are written in upstream raw order and the Rust harness compares
# undirected fixtures via canonicalised multisets.
RING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ring_py_path_p6_undirected",
        "origin": "python-igraph Graph.Ring(n=6, directed=False, "
        "mutual=False, circular=False)",
        "algo": "ring_graph",
        "params": {"n": 6, "directed": False, "mutual": False, "circular": False},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]],
        },
    },
    {
        "case": "ring_py_cycle_c6_undirected",
        "origin": "python-igraph Graph.Ring(n=6, directed=False, "
        "mutual=False, circular=True)",
        "algo": "ring_graph",
        "params": {"n": 6, "directed": False, "mutual": False, "circular": True},
        "expected": {
            "vcount": 6,
            "ecount": 6,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]],
        },
    },
    {
        "case": "ring_py_directed_path_p3",
        "origin": "python-igraph Graph.Ring(n=3, directed=True, "
        "mutual=False, circular=False)",
        "algo": "ring_graph",
        "params": {"n": 3, "directed": True, "mutual": False, "circular": False},
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": True,
            "edges": [[0, 1], [1, 2]],
        },
    },
    {
        "case": "ring_py_directed_mutual_path_p3",
        "origin": "python-igraph Graph.Ring(n=3, directed=True, "
        "mutual=True, circular=False) — mutual emits back-arcs in order",
        "algo": "ring_graph",
        "params": {"n": 3, "directed": True, "mutual": True, "circular": False},
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 1], [1, 0], [1, 2], [2, 1]],
        },
    },
]


STAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "star_py_undirected_k1_5",
        "origin": "python-igraph Graph.Star(n=6, mode='undirected') — "
        "K1,5 with vertex 0 as the centre",
        "algo": "star_graph",
        "params": {"n": 6, "mode": "Undirected", "center": 0},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "directed": False,
            "edges": [[1, 0], [2, 0], [3, 0], [4, 0], [5, 0]],
        },
    },
    {
        "case": "star_py_out_center_zero",
        "origin": "python-igraph Graph.Star(n=4, mode='out') — "
        "directed out-star, centre emits to every leaf",
        "algo": "star_graph",
        "params": {"n": 4, "mode": "Out", "center": 0},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": True,
            "edges": [[0, 1], [0, 2], [0, 3]],
        },
    },
    {
        "case": "star_py_in_center_zero",
        "origin": "python-igraph Graph.Star(n=4, mode='in') — "
        "directed in-star, every leaf emits to centre",
        "algo": "star_graph",
        "params": {"n": 4, "mode": "In", "center": 0},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": True,
            "edges": [[1, 0], [2, 0], [3, 0]],
        },
    },
    {
        "case": "star_py_mutual_center_one",
        "origin": "python-igraph Graph.Star(n=4, mode='mutual', center=1) — "
        "both arcs per leaf, forward arc (centre→leaf) first",
        "algo": "star_graph",
        "params": {"n": 4, "mode": "Mutual", "center": 1},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": True,
            "edges": [
                [1, 0], [0, 1], [1, 2], [2, 1], [1, 3], [3, 1],
            ],
        },
    },
]


WHEEL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "wheel_py_undirected_w5",
        "origin": "python-igraph Graph.Wheel(n=5, mode='undirected') — "
        "4 spokes + 4 rim edges; centre degree 4, rim degree 3",
        "algo": "wheel_graph",
        "params": {"n": 5, "mode": "Undirected", "center": 0},
        "expected": {
            "vcount": 5,
            "ecount": 8,
            "directed": False,
            "edges": [
                [1, 0], [2, 0], [3, 0], [4, 0],
                [1, 2], [2, 3], [3, 4], [4, 1],
            ],
        },
    },
    {
        "case": "wheel_py_out_w6_center_zero",
        "origin": "python-igraph Graph.Wheel(n=6, mode='out') — "
        "directed out-wheel, all arcs flow forward",
        "algo": "wheel_graph",
        "params": {"n": 6, "mode": "Out", "center": 0},
        "expected": {
            "vcount": 6,
            "ecount": 10,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4], [0, 5],
                [1, 2], [2, 3], [3, 4], [4, 5], [5, 1],
            ],
        },
    },
    {
        "case": "wheel_py_in_w6_center_zero",
        "origin": "python-igraph Graph.Wheel(n=6, mode='in') — "
        "directed in-wheel, spokes leaf→centre",
        "algo": "wheel_graph",
        "params": {"n": 6, "mode": "In", "center": 0},
        "expected": {
            "vcount": 6,
            "ecount": 10,
            "directed": True,
            "edges": [
                [1, 0], [2, 0], [3, 0], [4, 0], [5, 0],
                [1, 2], [2, 3], [3, 4], [4, 5], [5, 1],
            ],
        },
    },
    {
        "case": "wheel_py_three_vertex_parallel_rim",
        "origin": "python-igraph Graph.Wheel(n=3, mode='out') — "
        "degenerate: rim collapses to 2-cycle, parallel edges (1,2) and (2,1)",
        "algo": "wheel_graph",
        "params": {"n": 3, "mode": "Out", "center": 0},
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 1], [0, 2], [1, 2], [2, 1]],
        },
    },
]

KARY_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "kary_tree_py_binary_seven_undirected",
        "origin": "python-igraph Graph.Tree(n=7, children=2, mode='undirected') — "
        "perfect binary tree depth 2, undirected",
        "algo": "kary_tree",
        "params": {"n": 7, "children": 2, "mode": "Undirected"},
        "expected": {
            "vcount": 7,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [1, 3], [1, 4], [2, 5], [2, 6],
            ],
        },
    },
    {
        "case": "kary_tree_py_binary_seven_out",
        "origin": "python-igraph Graph.Tree(n=7, children=2, mode='out') — "
        "perfect binary tree, parent→child arcs",
        "algo": "kary_tree",
        "params": {"n": 7, "children": 2, "mode": "Out"},
        "expected": {
            "vcount": 7,
            "ecount": 6,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [1, 3], [1, 4], [2, 5], [2, 6],
            ],
        },
    },
    {
        "case": "kary_tree_py_ternary_eight_partial",
        "origin": "python-igraph Graph.Tree(n=8, children=3, mode='out') — "
        "ternary tree where last parent has only one child",
        "algo": "kary_tree",
        "params": {"n": 8, "children": 3, "mode": "Out"},
        "expected": {
            "vcount": 8,
            "ecount": 7,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3], [1, 4], [1, 5], [1, 6], [2, 7],
            ],
        },
    },
    {
        "case": "kary_tree_py_quaternary_thirteen_undirected",
        "origin": "python-igraph Graph.Tree(n=13, children=4, mode='undirected') — "
        "depth-2 quaternary tree (1 + 4 + 8 = 13 vertices)",
        "algo": "kary_tree",
        "params": {"n": 13, "children": 4, "mode": "Undirected"},
        "expected": {
            "vcount": 13,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4],
                [1, 5], [1, 6], [1, 7], [1, 8],
                [2, 9], [2, 10], [2, 11], [2, 12],
            ],
        },
    },
]

SYMMETRIC_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "symmetric_tree_py_binary_two_two_undirected",
        "origin": "python-igraph Graph.SymmetricTree([2, 2], 'undirected') — "
        "1 + 2 + 4 = 7 vertices, equivalent to Graph.Tree(7, 2)",
        "algo": "symmetric_tree",
        "params": {"branches": [2, 2], "mode": "Undirected"},
        "expected": {
            "vcount": 7,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [1, 3], [1, 4], [2, 5], [2, 6],
            ],
        },
    },
    {
        "case": "symmetric_tree_py_three_two_out",
        "origin": "python-igraph Graph.SymmetricTree([3, 2], 'out') — "
        "1 + 3 + 6 = 10 vertices, parent→child arcs",
        "algo": "symmetric_tree",
        "params": {"branches": [3, 2], "mode": "Out"},
        "expected": {
            "vcount": 10,
            "ecount": 9,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 4], [1, 5], [2, 6], [2, 7], [3, 8], [3, 9],
            ],
        },
    },
    {
        "case": "symmetric_tree_py_chain_three_ones_undirected",
        "origin": "python-igraph Graph.SymmetricTree([1, 1, 1], 'undirected') — "
        "linear chain of 4 vertices (BFS path)",
        "algo": "symmetric_tree",
        "params": {"branches": [1, 1, 1], "mode": "Undirected"},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3]],
        },
    },
    {
        "case": "symmetric_tree_py_four_three_two_out",
        "origin": "python-igraph Graph.SymmetricTree([4, 3, 2], 'out') — "
        "1 + 4 + 12 + 24 = 41 vertices, depth-3 mixed branching",
        "algo": "symmetric_tree",
        "params": {"branches": [4, 3, 2], "mode": "Out"},
        "expected": {
            "vcount": 41,
            "ecount": 40,
            "directed": True,
            "edges": [
                # level 0 → 1: root expands [4] kids
                [0, 1], [0, 2], [0, 3], [0, 4],
                # level 1 → 2: each of vertices 1..=4 expands [3] kids
                [1, 5], [1, 6], [1, 7],
                [2, 8], [2, 9], [2, 10],
                [3, 11], [3, 12], [3, 13],
                [4, 14], [4, 15], [4, 16],
                # level 2 → 3: each of vertices 5..=16 expands [2] kids
                [5, 17], [5, 18], [6, 19], [6, 20], [7, 21], [7, 22],
                [8, 23], [8, 24], [9, 25], [9, 26], [10, 27], [10, 28],
                [11, 29], [11, 30], [12, 31], [12, 32], [13, 33], [13, 34],
                [14, 35], [14, 36], [15, 37], [15, 38], [16, 39], [16, 40],
            ],
        },
    },
]


REGULAR_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "regular_tree_py_h2_k3_out",
        "origin": "python-igraph Graph.Regular_Tree(2, 3, 'out') — "
        "Bethe lattice h=2 k=3 (branches=[3,2]); 1+3+6=10 vertices",
        "algo": "regular_tree",
        "params": {"h": 2, "k": 3, "mode": "Out"},
        "expected": {
            "vcount": 10,
            "ecount": 9,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 4], [1, 5], [2, 6], [2, 7], [3, 8], [3, 9],
            ],
        },
    },
    {
        "case": "regular_tree_py_h1_k4_undirected",
        "origin": "python-igraph Graph.Regular_Tree(1, 4, 'undirected') — "
        "h=1 k=4 (branches=[4]); equivalent to star K1,4",
        "algo": "regular_tree",
        "params": {"h": 1, "k": 4, "mode": "Undirected"},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4],
            ],
        },
    },
    {
        "case": "regular_tree_py_h3_k3_out",
        "origin": "python-igraph Graph.Regular_Tree(3, 3, 'out') — "
        "Bethe lattice h=3 k=3 (branches=[3,2,2]); 1+3+6+12=22 vertices",
        "algo": "regular_tree",
        "params": {"h": 3, "k": 3, "mode": "Out"},
        "expected": {
            "vcount": 22,
            "ecount": 21,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 4], [1, 5], [2, 6], [2, 7], [3, 8], [3, 9],
                [4, 10], [4, 11], [5, 12], [5, 13],
                [6, 14], [6, 15], [7, 16], [7, 17],
                [8, 18], [8, 19], [9, 20], [9, 21],
            ],
        },
    },
    {
        "case": "regular_tree_py_h2_k2_undirected",
        "origin": "python-igraph Graph.Regular_Tree(2, 2, 'undirected') — "
        "degenerate k=2 case (branches=[2,1]); 1+2+2=5 vertices, P5 shape",
        "algo": "regular_tree",
        "params": {"h": 2, "k": 2, "mode": "Undirected"},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [1, 3], [2, 4],
            ],
        },
    },
]


HYPERCUBE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hypercube_py_n1_undirected",
        "origin": "python-igraph Graph.Hypercube(1, directed=False) — Q_1 = K_2",
        "algo": "hypercube",
        "params": {"n": 1, "directed": False},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
    {
        "case": "hypercube_py_n2_undirected",
        "origin": "python-igraph Graph.Hypercube(2, directed=False) — 4-cycle Q_2",
        "algo": "hypercube",
        "params": {"n": 2, "directed": False},
        "expected": {
            "vcount": 4,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 3], [2, 3]],
        },
    },
    {
        "case": "hypercube_py_n3_undirected",
        "origin": "python-igraph Graph.Hypercube(3, directed=False) — 8-vertex cube Q_3",
        "algo": "hypercube",
        "params": {"n": 3, "directed": False},
        "expected": {
            "vcount": 8,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 4],
                [1, 3], [1, 5],
                [2, 3], [2, 6],
                [3, 7],
                [4, 5], [4, 6],
                [5, 7],
                [6, 7],
            ],
        },
    },
    {
        "case": "hypercube_py_n3_directed",
        "origin": "python-igraph Graph.Hypercube(3, directed=True) — Q_3 oriented low->high",
        "algo": "hypercube",
        "params": {"n": 3, "directed": True},
        "expected": {
            "vcount": 8,
            "ecount": 12,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 4],
                [1, 3], [1, 5],
                [2, 3], [2, 6],
                [3, 7],
                [4, 5], [4, 6],
                [5, 7],
                [6, 7],
            ],
        },
    },
]


GENERALIZED_PETERSEN_MANIFEST: List[Dict[str, Any]] = [
    # Note: python-igraph does not expose `Graph.GeneralizedPetersen`
    # directly. We use Graph.Famous('Petersen') as the only available
    # canonical G(n,k) labeling — Famous('Dodecahedron') is also
    # isomorphic to G(10,2) but uses an embedded-polytope vertex layout
    # whose edge multiset differs from the canonical one, so we cannot
    # include it without isomorphism-based comparison.
    {
        "case": "generalized_petersen_py_g_5_2_petersen",
        "origin": "python-igraph Graph.Famous('Petersen') — the classic Petersen graph G(5,2)",
        "algo": "generalized_petersen",
        "params": {"n": 5, "k": 2},
        "expected": {
            "vcount": 10,
            "ecount": 15,
            "directed": False,
            "edges": [
                [0, 1], [0, 4], [0, 5],
                [1, 2], [1, 6],
                [2, 3], [2, 7],
                [3, 4], [3, 8],
                [4, 9],
                [5, 7], [5, 8],
                [6, 8], [6, 9],
                [7, 9],
            ],
        },
    },
]


CIRCULANT_MANIFEST: List[Dict[str, Any]] = [
    # Note: python-igraph 0.11.x does not expose `Graph.Circulant`
    # directly. We cover the two canonical specializations whose Famous /
    # constructor forms map onto a single-shift / full-shift-set
    # circulant: Graph.Ring(n, circular=True) ≡ circulant(n, [1], false)
    # and Graph.Famous('Tetrahedral') ≡ circulant(4, [1, 2], false) = K_4.
    {
        "case": "circulant_py_c_5_shifts_1_ring",
        "origin": "python-igraph Graph.Ring(n=5, circular=True) — equivalent to circulant(5, [1], False) = C_5",
        "algo": "circulant",
        "params": {"n": 5, "shifts": [1], "directed": False},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [0, 4],
            ],
        },
    },
    {
        "case": "circulant_py_k4_shifts_1_2_tetrahedral",
        "origin": "python-igraph Graph.Famous('Tetrahedral') — equivalent to circulant(4, [1, 2], False) = K_4",
        "algo": "circulant",
        "params": {"n": 4, "shifts": [1, 2], "directed": False},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 3], [1, 3], [2, 3], [0, 1], [1, 2], [0, 2],
            ],
        },
    },
]


DE_BRUIJN_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph exposes `Graph.De_Bruijn(m, n)` directly. Edge
    # emission order matches the upstream C exactly: for each vertex
    # i ∈ [0, m^n), arcs (i, (i*m mod vcount) + b) for b ∈ [0, m).
    {
        "case": "de_bruijn_py_b_2_2",
        "origin": "python-igraph Graph.De_Bruijn(m=2, n=2) — 4 vertices, 8 directed arcs",
        "algo": "de_bruijn",
        "params": {"m": 2, "n": 2},
        "expected": {
            "vcount": 4,
            "ecount": 8,
            "directed": True,
            "edges": [
                [0, 0], [0, 1],
                [1, 2], [1, 3],
                [2, 0], [2, 1],
                [3, 2], [3, 3],
            ],
        },
    },
    {
        "case": "de_bruijn_py_b_3_2",
        "origin": "python-igraph Graph.De_Bruijn(m=3, n=2) — 9 vertices, 27 directed arcs",
        "algo": "de_bruijn",
        "params": {"m": 3, "n": 2},
        "expected": {
            "vcount": 9,
            "ecount": 27,
            "directed": True,
            "edges": [
                [0, 0], [0, 1], [0, 2],
                [1, 3], [1, 4], [1, 5],
                [2, 6], [2, 7], [2, 8],
                [3, 0], [3, 1], [3, 2],
                [4, 3], [4, 4], [4, 5],
                [5, 6], [5, 7], [5, 8],
                [6, 0], [6, 1], [6, 2],
                [7, 3], [7, 4], [7, 5],
                [8, 6], [8, 7], [8, 8],
            ],
        },
    },
]


KAUTZ_MANIFEST: List[Dict[str, Any]] = [
    # python-igraph exposes `Graph.Kautz(m, n)` directly, which dispatches
    # to the same upstream `igraph_kautz()` C entry point. Edge lists below
    # were generated by calling python-igraph and cross-checked against
    # this crate's `kautz(m, n)`.
    {
        "case": "kautz_py_m2_n1",
        "origin": "python-igraph Graph.Kautz(m=2, n=1) — 6 vertices, 12 directed arcs",
        "algo": "kautz",
        "params": {"m": 2, "n": 1},
        "expected": {
            "vcount": 6,
            "ecount": 12,
            "directed": True,
            "edges": [
                [0, 2], [0, 3],
                [1, 4], [1, 5],
                [2, 0], [2, 1],
                [3, 4], [3, 5],
                [4, 0], [4, 1],
                [5, 2], [5, 3],
            ],
        },
    },
    {
        "case": "kautz_py_m3_n2",
        "origin": "python-igraph Graph.Kautz(m=3, n=2) — 36 vertices, 108 directed arcs",
        "algo": "kautz",
        "params": {"m": 3, "n": 2},
        "expected": {
            "vcount": 36,
            "ecount": 108,
            "directed": True,
            "edges": [
                [0, 9], [0, 10], [0, 11],
                [1, 12], [1, 13], [1, 14],
                [2, 15], [2, 16], [2, 17],
                [3, 18], [3, 19], [3, 20],
                [4, 21], [4, 22], [4, 23],
                [5, 24], [5, 25], [5, 26],
                [6, 27], [6, 28], [6, 29],
                [7, 30], [7, 31], [7, 32],
                [8, 33], [8, 34], [8, 35],
                [9, 0], [9, 1], [9, 2],
                [10, 3], [10, 4], [10, 5],
                [11, 6], [11, 7], [11, 8],
                [12, 18], [12, 19], [12, 20],
                [13, 21], [13, 22], [13, 23],
                [14, 24], [14, 25], [14, 26],
                [15, 27], [15, 28], [15, 29],
                [16, 30], [16, 31], [16, 32],
                [17, 33], [17, 34], [17, 35],
                [18, 0], [18, 1], [18, 2],
                [19, 3], [19, 4], [19, 5],
                [20, 6], [20, 7], [20, 8],
                [21, 9], [21, 10], [21, 11],
                [22, 12], [22, 13], [22, 14],
                [23, 15], [23, 16], [23, 17],
                [24, 27], [24, 28], [24, 29],
                [25, 30], [25, 31], [25, 32],
                [26, 33], [26, 34], [26, 35],
                [27, 0], [27, 1], [27, 2],
                [28, 3], [28, 4], [28, 5],
                [29, 6], [29, 7], [29, 8],
                [30, 9], [30, 10], [30, 11],
                [31, 12], [31, 13], [31, 14],
                [32, 15], [32, 16], [32, 17],
                [33, 18], [33, 19], [33, 20],
                [34, 21], [34, 22], [34, 23],
                [35, 24], [35, 25], [35, 26],
            ],
        },
    },
]


SQUARE_LATTICE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "square_lattice_py_dim_three_path",
        "origin": "python-igraph Graph.Lattice([3], nei=1, circular=False) — path P_3",
        "algo": "square_lattice",
        "params": {
            "dim": [3],
            "nei": 1,
            "directed": False,
            "mutual": False,
            "periodic": [False],
        },
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 1], [1, 2]],
        },
    },
    {
        "case": "square_lattice_py_dim_3x3_grid",
        "origin": "python-igraph Graph.Lattice([3, 3], nei=1, circular=False) — 3x3 grid, 12 e",
        "algo": "square_lattice",
        "params": {
            "dim": [3, 3],
            "nei": 1,
            "directed": False,
            "mutual": False,
            "periodic": None,
        },
        "expected": {
            "vcount": 9,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 3],
                [1, 2], [1, 4],
                [2, 5],
                [3, 4], [3, 6],
                [4, 5], [4, 7],
                [5, 8],
                [6, 7],
                [7, 8],
            ],
        },
    },
    {
        "case": "square_lattice_py_dim_3x3_torus",
        "origin": "python-igraph Graph.Lattice([3, 3], nei=1, circular=True) — 3x3 torus, 18 e",
        "algo": "square_lattice",
        "params": {
            "dim": [3, 3],
            "nei": 1,
            "directed": False,
            "mutual": False,
            "periodic": [True, True],
        },
        "expected": {
            "vcount": 9,
            "ecount": 18,
            "directed": False,
            "edges": [
                [0, 1], [0, 3],
                [1, 2], [1, 4],
                [0, 2], [2, 5],
                [3, 4], [3, 6],
                [4, 5], [4, 7],
                [3, 5], [5, 8],
                [6, 7], [0, 6],
                [7, 8], [1, 7],
                [6, 8], [2, 8],
            ],
        },
    },
    {
        "case": "square_lattice_py_dim_2x2x2_cube",
        "origin": "python-igraph Graph.Lattice([2, 2, 2], nei=1, circular=False) — Q_3 cube",
        "algo": "square_lattice",
        "params": {
            "dim": [2, 2, 2],
            "nei": 1,
            "directed": False,
            "mutual": False,
            "periodic": None,
        },
        "expected": {
            "vcount": 8,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 4],
                [1, 3], [1, 5],
                [2, 3], [2, 6],
                [3, 7],
                [4, 5], [4, 6],
                [5, 7],
                [6, 7],
            ],
        },
    },
]


HAMMING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hamming_py_n1_q3_is_k3",
        "origin": "python-igraph Graph.Hamming(1, 3, directed=False) — K_3",
        "algo": "hamming",
        "params": {"n": 1, "q": 3, "directed": False},
        "expected": {
            "vcount": 3,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 2]],
        },
    },
    {
        "case": "hamming_py_n2_q3_undirected",
        "origin": "python-igraph Graph.Hamming(2, 3, directed=False) — H(2,3), 18 edges",
        "algo": "hamming",
        "params": {"n": 2, "q": 3, "directed": False},
        "expected": {
            "vcount": 9,
            "ecount": 18,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 6],
                [1, 2], [1, 4], [1, 7],
                [2, 5], [2, 8],
                [3, 4], [3, 5], [3, 6],
                [4, 5], [4, 7],
                [5, 8],
                [6, 7], [6, 8],
                [7, 8],
            ],
        },
    },
    {
        "case": "hamming_py_n3_q2_equals_hypercube_q3",
        "origin": "python-igraph Graph.Hamming(3, 2, directed=False) — H(3,2) ≡ Q_3",
        "algo": "hamming",
        "params": {"n": 3, "q": 2, "directed": False},
        "expected": {
            "vcount": 8,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 4],
                [1, 3], [1, 5],
                [2, 3], [2, 6],
                [3, 7],
                [4, 5], [4, 6],
                [5, 7],
                [6, 7],
            ],
        },
    },
    {
        "case": "hamming_py_n2_q3_directed",
        "origin": "python-igraph Graph.Hamming(2, 3, directed=True) — H(2,3) low->high arcs",
        "algo": "hamming",
        "params": {"n": 2, "q": 3, "directed": True},
        "expected": {
            "vcount": 9,
            "ecount": 18,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 6],
                [1, 2], [1, 4], [1, 7],
                [2, 5], [2, 8],
                [3, 4], [3, 5], [3, 6],
                [4, 5], [4, 7],
                [5, 8],
                [6, 7], [6, 8],
                [7, 8],
            ],
        },
    },
]


FULL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_py_n4_ud_noloops",
        "origin": "mirrors python-igraph Graph.Full(4, directed=False, loops=False) — undirected K_4, 6 edges (dispatches to igraph_full)",
        "algo": "full_graph",
        "params": {"n": 4, "directed": False, "loops": False},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 2], [1, 3],
                [2, 3],
            ],
        },
    },
    {
        "case": "full_py_n4_ud_loops",
        "origin": "mirrors python-igraph Graph.Full(4, directed=False, loops=True) — undirected K_4 + self-loops, 10 edges",
        "algo": "full_graph",
        "params": {"n": 4, "directed": False, "loops": True},
        "expected": {
            "vcount": 4,
            "ecount": 10,
            "directed": False,
            "edges": [
                [0, 0], [0, 1], [0, 2], [0, 3],
                [1, 1], [1, 2], [1, 3],
                [2, 2], [2, 3],
                [3, 3],
            ],
        },
    },
    {
        "case": "full_py_n3_d_noloops",
        "origin": "mirrors python-igraph Graph.Full(3, directed=True, loops=False) — directed K_3, 6 arcs",
        "algo": "full_graph",
        "params": {"n": 3, "directed": True, "loops": False},
        "expected": {
            "vcount": 3,
            "ecount": 6,
            "directed": True,
            "edges": [
                [0, 1], [0, 2],
                [1, 0], [1, 2],
                [2, 0], [2, 1],
            ],
        },
    },
    {
        "case": "full_py_n3_d_loops",
        "origin": "mirrors python-igraph Graph.Full(3, directed=True, loops=True) — directed K_3 + self-loops, 9 arcs (n^2)",
        "algo": "full_graph",
        "params": {"n": 3, "directed": True, "loops": True},
        "expected": {
            "vcount": 3,
            "ecount": 9,
            "directed": True,
            "edges": [
                [0, 0], [0, 1], [0, 2],
                [1, 0], [1, 1], [1, 2],
                [2, 0], [2, 1], [2, 2],
            ],
        },
    },
]


# ALGO-CN-025 — `Graph.Full_Citation(n, directed=False)` (Python bindings,
# dispatches to `igraph_full_citation`). The python-igraph testFullCitation
# (`tests/test_generators.py:120`) asserts the *sorted* edge lists match
# the closed-form `[(x, y) for x in range(n) for y in range(x+1, n)]` (or
# its descending counterpart for the directed case). Our manifest carries
# the *emission* order produced by `igraph_full_citation` itself —
# descending-source-major `(i, j)` with `j < i` — so the conformance
# comparator runs over the canonical-undirected multiset (consistent with
# how the upstream test sorts before comparing).
FULL_CITATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_citation_py_n6_undirected",
        "origin": "mirrors python-igraph Graph.Full_Citation(6) — undirected K_6 (testFullCitation case 1, scaled down for fixture readability)",
        "algo": "full_citation",
        "params": {"n": 6, "directed": False},
        "expected": {
            "vcount": 6,
            "ecount": 15,
            "directed": False,
            "edges": [
                [1, 0],
                [2, 0], [2, 1],
                [3, 0], [3, 1], [3, 2],
                [4, 0], [4, 1], [4, 2], [4, 3],
                [5, 0], [5, 1], [5, 2], [5, 3], [5, 4],
            ],
        },
    },
    {
        "case": "full_citation_py_n6_directed",
        "origin": "mirrors python-igraph Graph.Full_Citation(6, True) — complete DAG with arcs i->j for every j<i (testFullCitation case 2, scaled to n=6)",
        "algo": "full_citation",
        "params": {"n": 6, "directed": True},
        "expected": {
            "vcount": 6,
            "ecount": 15,
            "directed": True,
            "edges": [
                [1, 0],
                [2, 0], [2, 1],
                [3, 0], [3, 1], [3, 2],
                [4, 0], [4, 1], [4, 2], [4, 3],
                [5, 0], [5, 1], [5, 2], [5, 3], [5, 4],
            ],
        },
    },
    {
        "case": "full_citation_py_n2_directed_single_arc",
        "origin": "mirrors python-igraph Graph.Full_Citation(2, True) — degenerate smallest non-trivial DAG with a single arc 1->0",
        "algo": "full_citation",
        "params": {"n": 2, "directed": True},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": True,
            "edges": [[1, 0]],
        },
    },
]


# ALGO-CN-026 — `Graph.Full_Bipartite` / `Graph.Full_Multipartite` (Python
# bindings, dispatches to `igraph_full_multipartite`). python-igraph's
# `Graph.Full_Bipartite(n1, n2)` is the canonical bipartite shorthand
# (partitions=[n1, n2]); `Graph.Full_Multipartite(n_list, directed, mode)`
# is the general entry point introduced upstream. The Python tests in
# `tests/test_generators.py` assert the constructor returns the expected
# vertex / edge count and partition `types` vector. Our manifest carries
# three fixtures: a canonical undirected K_{3,4} bipartite, the
# directed-OUT version of the same partitions, and a small tripartite
# K_{1,2,2} mutual case. Comparison is multiset-based.
FULL_MULTIPARTITE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_multipartite_py_k34_undirected",
        "origin": "mirrors python-igraph Graph.Full_Bipartite(3, 4) — undirected K_{3,4} with 12 edges and types=[0,0,0,1,1,1,1]",
        "algo": "full_multipartite",
        "params": {"partitions": [3, 4], "directed": False, "mode": "all"},
        "expected": {
            "vcount": 7,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 3], [0, 4], [0, 5], [0, 6],
                [1, 3], [1, 4], [1, 5], [1, 6],
                [2, 3], [2, 4], [2, 5], [2, 6],
            ],
            "types": [0, 0, 0, 1, 1, 1, 1],
        },
    },
    {
        "case": "full_multipartite_py_k34_directed_out",
        "origin": "mirrors python-igraph Graph.Full_Bipartite(3, 4, directed=True, mode='OUT') — 12 arcs flowing from partition 0 → partition 1",
        "algo": "full_multipartite",
        "params": {"partitions": [3, 4], "directed": True, "mode": "out"},
        "expected": {
            "vcount": 7,
            "ecount": 12,
            "directed": True,
            "edges": [
                [0, 3], [0, 4], [0, 5], [0, 6],
                [1, 3], [1, 4], [1, 5], [1, 6],
                [2, 3], [2, 4], [2, 5], [2, 6],
            ],
            "types": [0, 0, 0, 1, 1, 1, 1],
        },
    },
    {
        "case": "full_multipartite_py_tripartite_1_2_2_directed_all",
        "origin": "mirrors python-igraph Graph.Full_Multipartite([1,2,2], directed=True, mode='ALL') — K_{1,2,2} with 16 mutual arcs (= 2 · 8 undirected edges)",
        "algo": "full_multipartite",
        "params": {"partitions": [1, 2, 2], "directed": True, "mode": "all"},
        "expected": {
            "vcount": 5,
            "ecount": 16,
            "directed": True,
            "edges": [
                [0, 1], [1, 0], [0, 2], [2, 0], [0, 3], [3, 0], [0, 4], [4, 0],
                [1, 3], [3, 1], [1, 4], [4, 1], [2, 3], [3, 2], [2, 4], [4, 2],
            ],
            "types": [0, 1, 1, 2, 2],
        },
    },
]


# ALGO-CN-015 — `Graph.linegraph()` (Python bindings, dispatches to the
# same C `igraph_linegraph`). Fixtures focus on small textbook shapes
# (P_4, K_4, C_5) plus a directed cycle that exercises chain semantics.
LINEGRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "linegraph_py_path_p4_undirected",
        "origin": "python-igraph Graph.linegraph() on the path P_4 (0-1, 1-2, 2-3) → P_3 on three L-vertices",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "params": {},
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 1], [1, 2]],
        },
    },
    {
        "case": "linegraph_py_complete_k4_undirected",
        "origin": "python-igraph Graph.linegraph() on K_4 — six L-vertices, 12 L-edges (every edge shares an endpoint with four others)",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            4,
            edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "params": {},
        "expected": {
            "vcount": 6,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [1, 2], [1, 3], [0, 3], [2, 4],
                [0, 4], [3, 4], [2, 5], [4, 5], [1, 5], [3, 5],
            ],
        },
    },
    {
        "case": "linegraph_py_cycle_c5_undirected",
        "origin": "python-igraph Graph.linegraph() on the 5-cycle C_5 → C_5 (cycle of length n preserves length under L)",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            5,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4), (0, 4)],
            directed=False,
        ),
        "params": {},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [0, 4]],
        },
    },
    {
        "case": "linegraph_py_path_p4_directed",
        "origin": "python-igraph Graph.linegraph() on directed P_4 (arcs 0→1→2→3) → directed P_3",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "params": {},
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": True,
            "edges": [[0, 1], [1, 2]],
        },
    },
    {
        "case": "linegraph_py_directed_3cycle",
        "origin": "python-igraph Graph.linegraph() on a directed 3-cycle (0→1→2→0) → directed 3-cycle on its three L-vertices",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "params": {},
        "expected": {
            "vcount": 3,
            "ecount": 3,
            "directed": True,
            "edges": [[2, 0], [0, 1], [1, 2]],
        },
    },
]


# ALGO-CN-016 — `Graph.Prufer(seq)` (Python bindings; dispatches to the
# same C `igraph_from_prufer`). Fixtures span: empty (P_2), single-entry,
# repeated-vertex (star), ascending (path), and an arbitrary mixed
# sequence — five distinct topologies so cross-source ordering doesn't
# rely on any one canonicalisation.
# ALGO-CN-017 — python-igraph does NOT expose a direct binding for
# `igraph_tree_from_parent_vector` (no `Graph.TreeFromParentVector`
# class method), so these fixtures are synthesised from the same C-level
# semantics — chosen sequences that python users would naturally build
# by walking a `predecessors`/`predecessor_id` vector from
# `Graph.bfs(...)` or `Graph.shortest_paths(...)` output. Each fixture
# would round-trip through any future python binding as
# `Graph(n, [(parent, child) for child, parent in enumerate(parents) if parent >= 0],
#         directed=True)` in OUT mode.
TREE_FROM_PARENT_VECTOR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "tree_from_parent_vector_py_chain_undirected",
        "origin": "python-side equivalent — parents=[-1,0,1,2,3] (chain 0-1-2-3-4) decoded undirected yields path P_5",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-1, 0, 1, 2, 3], "mode": "undirected"},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4]],
        },
    },
    {
        "case": "tree_from_parent_vector_py_star_out",
        "origin": "python-side equivalent — parents=[-1,0,0,0,0] (star centred at 0) decoded OUT yields directed star with edges 0→{1,2,3,4}",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-1, 0, 0, 0, 0], "mode": "out"},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 1], [0, 2], [0, 3], [0, 4]],
        },
    },
    {
        "case": "tree_from_parent_vector_py_star_in",
        "origin": "python-side equivalent — parents=[-1,0,0,0,0] decoded IN yields edges {1,2,3,4}→0 (inverted star)",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-1, 0, 0, 0, 0], "mode": "in"},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": True,
            "edges": [[1, 0], [2, 0], [3, 0], [4, 0]],
        },
    },
    {
        "case": "tree_from_parent_vector_py_two_root_forest",
        "origin": "python-side equivalent — parents=[-1,0,-1,2,3] (two roots: 0 and 2) decoded OUT yields two paths 0→1 and 2→3→4",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-1, 0, -1, 2, 3], "mode": "out"},
        "expected": {
            "vcount": 5,
            "ecount": 3,
            "directed": True,
            "edges": [[0, 1], [2, 3], [3, 4]],
        },
    },
    {
        "case": "tree_from_parent_vector_py_singleton_root",
        "origin": "python-side equivalent — parents=[-1] (lone root vertex) decoded OUT yields directed K_1 (1 vertex, 0 edges)",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-1], "mode": "out"},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]


# ALGO-CN-018 — `Graph.LCF(n, shifts, repeats)` in python-igraph dispatches
# to the same C `igraph_lcf`. We mirror the upstream C bench fixtures and
# add canonical-LCF graphs from the standard cubic-graph catalogue
# (Frucht, Truncated tetrahedron, Truncated octahedron) — every entry is
# pinned to its canonical (sorted) edge list so cross-source comparison is
# deterministic.
LCF_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lcf_py_franklin",
        "origin": "python-igraph Graph.LCF(12, [5, -5], 6) — Franklin graph (12 vertices, 18 edges, bipartite cubic)",
        "algo": "lcf",
        "params": {"n": 12, "shifts": [5, -5], "repeats": 6},
        "expected": {
            "vcount": 12,
            "ecount": 18,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7],
                [7, 8], [8, 9], [9, 10], [10, 11], [0, 11],
                [0, 5], [1, 8], [2, 7], [3, 10], [4, 9], [6, 11],
            ],
        },
    },
    {
        "case": "lcf_py_heawood",
        "origin": "python-igraph Graph.LCF(14, [5, -5], 7) — Heawood graph (14 vertices, 21 edges, bipartite cubic, girth 6)",
        "algo": "lcf",
        "params": {"n": 14, "shifts": [5, -5], "repeats": 7},
        "expected": {
            "vcount": 14,
            "ecount": 21,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7],
                [7, 8], [8, 9], [9, 10], [10, 11], [11, 12], [12, 13], [0, 13],
                [0, 5], [1, 10], [2, 7], [3, 12], [4, 9], [6, 11], [8, 13],
            ],
        },
    },
    {
        "case": "lcf_py_truncated_tetrahedron",
        "origin": "python-igraph Graph.LCF(12, [2, 6, -2, -6], 3) — truncated tetrahedron (12 vertices, 18 edges, cubic)",
        "algo": "lcf",
        "params": {"n": 12, "shifts": [2, 6, -2, -6], "repeats": 3},
        "expected": {
            "vcount": 12,
            "ecount": 18,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7],
                [7, 8], [8, 9], [9, 10], [10, 11], [0, 11],
                [0, 2], [1, 7], [3, 9], [4, 6], [5, 11], [8, 10],
            ],
        },
    },
    {
        "case": "lcf_py_empty_shifts_pure_cycle",
        "origin": "python-igraph Graph.LCF(6, [], 0) — chord pass skipped; result is pure Hamilton cycle C_6",
        "algo": "lcf",
        "params": {"n": 6, "shifts": [], "repeats": 0},
        "expected": {
            "vcount": 6,
            "ecount": 6,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [0, 5]],
        },
    },
    {
        "case": "lcf_py_single_shift_repeats_3",
        "origin": "python-igraph Graph.LCF(6, [3], 6) — every vertex paired across the diameter; antipode chords collapse to 3 unique edges",
        "algo": "lcf",
        "params": {"n": 6, "shifts": [3], "repeats": 6},
        "expected": {
            "vcount": 6,
            "ecount": 9,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [0, 5],
                [0, 3], [1, 4], [2, 5],
            ],
        },
    },
]


# ALGO-CN-019 — python-igraph 0.11.x exposes neither `Graph.Mycielski`
# nor `Graph.Mycielskian`; the upstream C functions land in the next
# Cython binding update. Until then the "py" lane keeps the conformance
# corpus complete by mirroring the published Mycielski recurrence
# `(v', e') = (2v + 1, 3e + v)` plus the canonical small cases
# (M_3 = C_5, M_4 = Grötzsch). The rigraph snapshot (`r_*` fixtures)
# executes the same igraph C `igraph_mycielski_graph` and lands on the
# same edge multisets, so the cross-source check stays meaningful.
MYCIELSKI_GRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mycielski_graph_py_k3_c5",
        "origin": "no upstream binding in python-igraph 0.11.x; computed from M_3 = C_5 (the canonical first non-trivial Mycielski graph)",
        "algo": "mycielski_graph",
        "params": {"k": 3},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [0, 3], [1, 2], [2, 4], [3, 4]],
        },
    },
    {
        "case": "mycielski_graph_py_k4_grotzsch_counts",
        "origin": "no upstream binding in python-igraph 0.11.x; M_4 = Grötzsch graph (11 vertices, 20 edges, triangle-free, χ=4)",
        "algo": "mycielski_graph",
        "params": {"k": 4},
        "expected": {
            "vcount": 11,
            "ecount": 20,
            "directed": False,
            # Counts-only check (edges = null) — the structural recurrence
            # plus triangle-free property is what the literature pins down;
            # full edge list is exercised by the C and R lanes.
            "edges": None,
        },
    },
]


# python-igraph `Graph.Famous(name)` directly calls `igraph_famous` in
# the C core (see `_igraph/graphobject.c`). The expected blocks below
# were captured from python-igraph 0.11.9 so the rust port can be
# compared byte-for-byte against a live binding.
FAMOUS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "famous_py_bull",
        "origin": "python-igraph Graph.Famous('Bull') — 5v/5e small witness",
        "algo": "famous",
        "params": {"name": "Bull"},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 2], [1, 3], [2, 4]],
        },
    },
    {
        "case": "famous_py_petersen",
        "origin": "python-igraph Graph.Famous('Petersen') — 10v/15e Petersen",
        "algo": "famous",
        "params": {"name": "Petersen"},
        "expected": {
            "vcount": 10,
            "ecount": 15,
            "directed": False,
            "edges": [
                [0, 1], [0, 4], [0, 5], [1, 2], [1, 6],
                [2, 3], [2, 7], [3, 4], [3, 8], [4, 9],
                [5, 7], [5, 8], [6, 8], [6, 9], [7, 9],
            ],
        },
    },
    {
        "case": "famous_py_krackhardt_kite_lower",
        "origin": "python-igraph Graph.Famous('krackhardt_kite') — lowercase dispatch path on 10v/18e",
        "algo": "famous",
        "params": {"name": "krackhardt_kite"},
        "expected": {
            "vcount": 10,
            "ecount": 18,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 5], [1, 3], [1, 4], [1, 6],
                [2, 3], [2, 5], [3, 4], [3, 5], [3, 6],
                [4, 6], [5, 6], [5, 7], [6, 7], [7, 8], [8, 9],
            ],
        },
    },
    {
        "case": "famous_py_meredith_counts",
        "origin": "python-igraph Graph.Famous('Meredith') — 70v/140e largest entry; structural-only",
        "algo": "famous",
        "params": {"name": "Meredith"},
        "expected": {
            "vcount": 70,
            "ecount": 140,
            "directed": False,
            "edges": None,
        },
    },
    {
        "case": "famous_py_zachary_counts",
        "origin": "python-igraph Graph.Famous('Zachary') — 34v/78e karate club; structural-only",
        "algo": "famous",
        "params": {"name": "Zachary"},
        "expected": {
            "vcount": 34,
            "ecount": 78,
            "directed": False,
            "edges": None,
        },
    },
]


# python-igraph `Graph(edges, n=None, directed=False)` is the canonical
# `igraph_create` wrapper. Cases below exercise the same axes as the C
# fixtures (n-inference, n>max keeps, n<max extends, directed arc-order,
# empty, isolated, self/parallel) but produced via the Python binding so
# the JSON travels through `python-igraph`'s edge encoding.
CREATE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "create_py_simple_n_zero_infers_four",
        "origin": "python-igraph Graph([(0,1),(1,2),(2,3),(2,2)], directed=False) — vcount inferred",
        "algo": "create",
        "params": {
            "edges": [[0, 1], [1, 2], [2, 3], [2, 2]],
            "n": 0,
            "directed": False,
        },
        "expected": {
            "vcount": 4,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [2, 2]],
        },
    },
    {
        "case": "create_py_explicit_n_keeps_isolated",
        "origin": "python-igraph Graph([(0,1)], n=5, directed=False) — 5 vertices, 1 edge, isolated 2-4",
        "algo": "create",
        "params": {
            "edges": [[0, 1]],
            "n": 5,
            "directed": False,
        },
        "expected": {
            "vcount": 5,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
    {
        "case": "create_py_directed_arc_order",
        "origin": "python-igraph Graph([(0,1),(1,0)], n=2, directed=True) — both arcs distinct",
        "algo": "create",
        "params": {
            "edges": [[0, 1], [1, 0]],
            "n": 2,
            "directed": True,
        },
        "expected": {
            "vcount": 2,
            "ecount": 2,
            "directed": True,
            "edges": [[0, 1], [1, 0]],
        },
    },
    {
        "case": "create_py_empty_null",
        "origin": "python-igraph Graph([], n=0, directed=False) — null graph",
        "algo": "create",
        "params": {
            "edges": [],
            "n": 0,
            "directed": False,
        },
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "create_py_star_via_edges",
        "origin": "python-igraph Graph([(0,1),(0,2),(0,3),(0,4)], directed=False) — K_{1,4} star via create",
        "algo": "create",
        "params": {
            "edges": [[0, 1], [0, 2], [0, 3], [0, 4]],
            "n": 0,
            "directed": False,
        },
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 1], [0, 2], [0, 3], [0, 4]],
        },
    },
    {
        "case": "create_py_self_loop_present",
        "origin": "python-igraph Graph([(0,0),(0,1)], directed=False) — self-loop kept as edge",
        "algo": "create",
        "params": {
            "edges": [[0, 0], [0, 1]],
            "n": 0,
            "directed": False,
        },
        "expected": {
            "vcount": 2,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 0], [0, 1]],
        },
    },
]


# Fixtures for `Graph.Triangular_Lattice` (ALGO-CN-023). The three cases
# replicate the python-igraph test_generators.testTriangularLattice
# (`tests/test_generators.py:459-485`) which is the canonical lane
# checker for the constructor's edge-set contract: dims=[2,2] in three
# (directed, mutual) combinations.
TRIANGULAR_LATTICE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "triangular_lattice_py_2x2_undirected",
        "origin": "python-igraph Graph.Triangular_Lattice([2, 2]) — undirected default",
        "algo": "triangular_lattice",
        "params": {"dims": [2, 2], "directed": False, "mutual": False},
        "expected": {
            "vcount": 4,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [0, 2], [0, 3], [1, 3], [2, 3]],
        },
    },
    {
        "case": "triangular_lattice_py_2x2_directed_unilateral",
        "origin": "python-igraph Graph.Triangular_Lattice([2, 2], directed=True, mutual=False)",
        "algo": "triangular_lattice",
        "params": {"dims": [2, 2], "directed": True, "mutual": False},
        "expected": {
            "vcount": 4,
            "ecount": 5,
            "directed": True,
            "edges": [[0, 1], [0, 2], [0, 3], [1, 3], [2, 3]],
        },
    },
    {
        "case": "triangular_lattice_py_2x2_directed_mutual",
        "origin": "python-igraph Graph.Triangular_Lattice([2, 2], directed=True, mutual=True)",
        "algo": "triangular_lattice",
        "params": {"dims": [2, 2], "directed": True, "mutual": True},
        "expected": {
            "vcount": 4,
            "ecount": 10,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 0], [1, 3],
                [2, 0], [2, 3],
                [3, 0], [3, 1], [3, 2],
            ],
        },
    },
]


def _py_hex_lattice_expected(dims: List[int], directed: bool, mutual: bool) -> Dict[str, Any]:
    """Same idea as `_hex_lattice_expected` in `from_c.py` but anchored
    to the python-igraph wrapper `Graph.Hexagonal_Lattice`. Both
    converge on the same C core so the edges agree."""
    g = ig.Graph.Hexagonal_Lattice(dims, directed=directed, mutual=mutual)
    return {
        "vcount": g.vcount(),
        "ecount": g.ecount(),
        "directed": bool(g.is_directed()),
        "edges": [list(e.tuple) for e in g.es],
    }


# Fixtures for `Graph.Hexagonal_Lattice` (ALGO-CN-024). Mirrors the
# python-igraph `testHexagonalLattice` test
# (`tests/test_generators.py:135-164`) which is the canonical lane
# checker for the constructor's edge-set contract: dims=[2,2] in three
# (directed, mutual) combinations.
HEXAGONAL_LATTICE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hexagonal_lattice_py_2x2_undirected",
        "origin": "python-igraph Graph.Hexagonal_Lattice([2, 2]) — undirected default",
        "algo": "hexagonal_lattice",
        "params": {"dims": [2, 2], "directed": False, "mutual": False},
        "expected": _py_hex_lattice_expected([2, 2], False, False),
    },
    {
        "case": "hexagonal_lattice_py_2x2_directed_unilateral",
        "origin": "python-igraph Graph.Hexagonal_Lattice([2, 2], directed=True, mutual=False)",
        "algo": "hexagonal_lattice",
        "params": {"dims": [2, 2], "directed": True, "mutual": False},
        "expected": _py_hex_lattice_expected([2, 2], True, False),
    },
    {
        "case": "hexagonal_lattice_py_2x2_directed_mutual",
        "origin": "python-igraph Graph.Hexagonal_Lattice([2, 2], directed=True, mutual=True)",
        "algo": "hexagonal_lattice",
        "params": {"dims": [2, 2], "directed": True, "mutual": True},
        "expected": _py_hex_lattice_expected([2, 2], True, True),
    },
]


# python-igraph `Graph.Atlas(number)` calls `igraph_atlas` in the C core.
# Captured live from python-igraph 0.11.9 with the script:
#   for i in [0, 3, 18, 70, 180, 208, 1252]:
#       g = ig.Graph.Atlas(i); print(g.vcount(), g.ecount(), [e.tuple for e in g.es])
# Indices 70 and 180 are the ones python-igraph's own test_atlas.py drops
# from its connectivity sweep (line 174); including them here guards the
# *constructor* from regressions even though they're connectivity edge
# cases.
ATLAS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "atlas_py_null0",
        "origin": "python-igraph Graph.Atlas(0) — null graph on 0 vertices",
        "algo": "atlas",
        "params": {"number": 0},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "atlas_py_k2",
        "origin": "python-igraph Graph.Atlas(3) — single edge K_2",
        "algo": "atlas",
        "params": {"number": 3},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
    {
        "case": "atlas_py_k4",
        "origin": "python-igraph Graph.Atlas(18) — complete K_4",
        "algo": "atlas",
        "params": {"number": 18},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [[0, 1], [1, 2], [0, 2], [0, 3], [1, 3], [2, 3]],
        },
    },
    {
        "case": "atlas_py_idx70_skipped_in_upstream",
        "origin": "python-igraph Graph.Atlas(70) — 6v/4e graph dropped from upstream pagerank sweep; constructor itself is well-formed",
        "algo": "atlas",
        "params": {"number": 70},
        "expected": {
            "vcount": 6,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 2], [0, 4], [1, 3], [3, 5]],
        },
    },
    {
        "case": "atlas_py_idx180_skipped_in_upstream",
        "origin": "python-igraph Graph.Atlas(180) — 6v/10e graph dropped from upstream pagerank sweep",
        "algo": "atlas",
        "params": {"number": 180},
        "expected": {
            "vcount": 6,
            "ecount": 10,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [0, 4],
                [1, 3], [1, 4], [4, 5], [3, 5], [1, 5],
            ],
        },
    },
    {
        "case": "atlas_py_k6",
        "origin": "python-igraph Graph.Atlas(208) — complete K_6, last 6-vertex entry",
        "algo": "atlas",
        "params": {"number": 208},
        "expected": {
            "vcount": 6,
            "ecount": 15,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4], [0, 5],
                [1, 2], [1, 3], [1, 4], [1, 5],
                [2, 3], [2, 4], [2, 5],
                [3, 4], [3, 5],
                [4, 5],
            ],
        },
    },
    {
        "case": "atlas_py_k7_last",
        "origin": "python-igraph Graph.Atlas(1252) — complete K_7, last entry in the atlas",
        "algo": "atlas",
        "params": {"number": 1252},
        "expected": {
            "vcount": 7,
            "ecount": 21,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4], [0, 5], [0, 6],
                [1, 2], [1, 3], [1, 4], [1, 5], [1, 6],
                [2, 3], [2, 4], [2, 5], [2, 6],
                [3, 4], [3, 5], [3, 6],
                [4, 5], [4, 6],
                [5, 6],
            ],
        },
    },
]


MYCIELSKIAN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mycielskian_py_p3_one_iteration",
        "origin": "no upstream binding in python-igraph 0.11.x; mycielskian(P_3, k=1) → 7v/9e from the published recurrence",
        "graph_factory": lambda: ig.Graph(n=3, edges=[(0, 1), (1, 2)], directed=False),
        "algo": "mycielskian",
        "params": {"k": 1},
        "expected": {
            "vcount": 7,
            "ecount": 9,
            "directed": False,
            "edges": [
                [0, 1], [1, 2],
                [0, 4], [1, 3], [1, 5], [2, 4],
                [3, 6], [4, 6], [5, 6],
            ],
        },
    },
    {
        "case": "mycielskian_py_singleton_one_iteration_is_p2",
        "origin": "no upstream binding in python-igraph 0.11.x; mycielskian(singleton, k=1) promotes to P_2 (k=1 base case)",
        "graph_factory": lambda: ig.Graph(n=1, edges=[], directed=False),
        "algo": "mycielskian",
        "params": {"k": 1},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
]


PRUFER_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "from_prufer_py_empty_yields_p2",
        "origin": "python-igraph Graph.Prufer([]) → P_2 (2 vertices, 1 edge)",
        "algo": "from_prufer",
        "params": {"prufer": []},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
    {
        "case": "from_prufer_py_singleton",
        "origin": "python-igraph Graph.Prufer([0]) → 3-vertex tree centred at 0",
        "algo": "from_prufer",
        "params": {"prufer": [0]},
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 1], [0, 2]],
        },
    },
    {
        "case": "from_prufer_py_constant_star",
        "origin": "python-igraph Graph.Prufer([0,0,0,0]) → star S_6 centred at 0",
        "algo": "from_prufer",
        "params": {"prufer": [0, 0, 0, 0]},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [0, 2], [0, 3], [0, 4], [0, 5]],
        },
    },
    {
        "case": "from_prufer_py_ascending_path",
        "origin": "python-igraph Graph.Prufer([1,2,3,4]) → path P_6 on vertices 0-1-2-3-4-5",
        "algo": "from_prufer",
        "params": {"prufer": [1, 2, 3, 4]},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]],
        },
    },
    {
        "case": "from_prufer_py_mixed_seq",
        "origin": "python-igraph Graph.Prufer([0,2,4,1,1,0]) — mixed 6-entry seq → 8-vertex tree (same as C fixture 2)",
        "algo": "from_prufer",
        "params": {"prufer": [0, 2, 4, 1, 1, 0]},
        "expected": {
            "vcount": 8,
            "ecount": 7,
            "directed": False,
            "edges": [[0, 1], [0, 3], [0, 7], [1, 4], [1, 6], [2, 4], [2, 5]],
        },
    },
]


# ALGO-CN-029 — `Graph.Adjacency(matrix, mode='directed'|'undirected'|'max'|
# 'min'|'plus'|'upper'|'lower', loops='ignore'|'once'|'twice')` in python-igraph
# dispatches to the same C `igraph_adjacency`. Captured live from
# python-igraph 0.11.9 with the script:
#   M3 = [[4,2,0],[3,0,4],[0,5,6]]
#   for mode, loops in [('directed','ignore'), ('max','ignore'),
#                       ('upper','twice'), ('plus','once')]:
#       g = ig.Graph.Adjacency(M3, mode=mode, loops=loops)
#       print(mode, loops, g.vcount(), g.ecount(), sorted(e.tuple for e in g.es))
# Undirected edges canonicalised to (min, max) to match the Rust side's
# `Graph::add_edges` storage. Loop count for python's 'once'/'twice'
# matches the C semantics: 'twice' is halved (with TWICE→ONCE collapse for
# directed/upper/lower per the C dispatcher).
ADJACENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "adjacency_py_3x3_directed_ignore",
        "origin": "python-igraph Graph.Adjacency([[4,2,0],[3,0,4],[0,5,6]], mode='directed', loops='ignore') — 14 off-diagonal arcs (M3 from C tests)",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [3, 0, 4], [0, 5, 6]],
            "mode": "directed",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 14,
            "directed": True,
            "edges": [
                [0, 1], [0, 1],
                [1, 0], [1, 0], [1, 0],
                [1, 2], [1, 2], [1, 2], [1, 2],
                [2, 1], [2, 1], [2, 1], [2, 1], [2, 1],
            ],
        },
    },
    {
        "case": "adjacency_py_3x3_max_ignore",
        "origin": "python-igraph Graph.Adjacency(M3, mode='max', loops='ignore') — pair (i,j) gets max(A[i,j],A[j,i]); (0,1)=3, (1,2)=5 → 8 undirected edges",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [3, 0, 4], [0, 5, 6]],
            "mode": "max",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 8,
            "directed": False,
            "edges": [
                [0, 1], [0, 1], [0, 1],
                [1, 2], [1, 2], [1, 2], [1, 2], [1, 2],
            ],
        },
    },
    {
        "case": "adjacency_py_3x3_plus_once",
        "origin": "python-igraph Graph.Adjacency(M3, mode='plus', loops='once') — PLUS off-diag (0,1)=5, (1,2)=9; diag {4,0,6} = 10 loops",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [3, 0, 4], [0, 5, 6]],
            "mode": "plus",
            "loops": "once",
        },
        "expected": {
            "vcount": 3,
            "ecount": 24,
            "directed": False,
            "edges": [
                [0, 0], [0, 0], [0, 0], [0, 0],
                [0, 1], [0, 1], [0, 1], [0, 1], [0, 1],
                [1, 2], [1, 2], [1, 2], [1, 2], [1, 2], [1, 2], [1, 2], [1, 2], [1, 2],
                [2, 2], [2, 2], [2, 2], [2, 2], [2, 2], [2, 2],
            ],
        },
    },
    {
        "case": "adjacency_py_3x3_upper_twice_collapsed",
        "origin": "python-igraph Graph.Adjacency(M3, mode='upper', loops='twice') — UPPER collapses TWICE→ONCE → diag {4,0,6} stays as 10 loops; off-diag (0,1)=2, (1,2)=4",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [3, 0, 4], [0, 5, 6]],
            "mode": "upper",
            "loops": "twice",
        },
        "expected": {
            "vcount": 3,
            "ecount": 16,
            "directed": False,
            "edges": [
                [0, 0], [0, 0], [0, 0], [0, 0],
                [0, 1], [0, 1],
                [1, 2], [1, 2], [1, 2], [1, 2],
                [2, 2], [2, 2], [2, 2], [2, 2], [2, 2], [2, 2],
            ],
        },
    },
]


# ALGO-CN-030 — `Graph.Weighted_Adjacency(matrix, mode, loops)` is the
# real-valued sibling of `Graph.Adjacency`. python-igraph wraps the same
# C dispatch (`igraph_weighted_adjacency`), so we capture the same M3
# matrix family but with f64 weights instead of integer multiplicities.
# Edges are canonicalised to (min, max) for undirected variants to match
# Rust storage. Weights are returned in the **edge order** the python-igraph
# wrapper returns them (parallel to its `Graph.es["weight"]`), and we
# compare them order-agnostic via a sorted (edge, weight) pair list in the
# Rust harness.
WEIGHTED_ADJACENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "weighted_adjacency_py_3x3_directed_ignore",
        "origin": "python-igraph Graph.Weighted_Adjacency([[2.0,0.5,0],[1.5,0,2.0],[0,2.5,3.0]], mode='directed', loops='ignore') — 4 off-diagonal weighted arcs",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [1.5, 0.0, 2.0], [0.0, 2.5, 3.0]],
            "mode": "directed",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": True,
            "edges": [[1, 0], [0, 1], [2, 1], [1, 2]],
            "weights": [1.5, 0.5, 2.5, 2.0],
        },
    },
    {
        "case": "weighted_adjacency_py_3x3_max_ignore",
        "origin": "python-igraph Graph.Weighted_Adjacency(M3, mode='max', loops='ignore') — (0,1)=max(0.5,1.5)=1.5, (1,2)=max(2.0,2.5)=2.5; pair (0,2) zero so skipped",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [1.5, 0.0, 2.0], [0.0, 2.5, 3.0]],
            "mode": "max",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 1], [1, 2]],
            "weights": [1.5, 2.5],
        },
    },
    {
        "case": "weighted_adjacency_py_3x3_plus_once",
        "origin": "python-igraph Graph.Weighted_Adjacency(M3, mode='plus', loops='once') — PLUS off-diag sums + diag passes through unhalved",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [1.5, 0.0, 2.0], [0.0, 2.5, 3.0]],
            "mode": "plus",
            "loops": "once",
        },
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": False,
            # row-major upper triangle: i=0 j=0 diag 2.0; i=0 j=1 (0,1)=0.5+1.5=2.0;
            # i=1 j=2 (1,2)=2.0+2.5=4.5; i=2 j=2 diag 3.0
            "edges": [[0, 0], [0, 1], [1, 2], [2, 2]],
            "weights": [2.0, 2.0, 4.5, 3.0],
        },
    },
    {
        "case": "weighted_adjacency_py_3x3_upper_twice_collapsed",
        "origin": "python-igraph Graph.Weighted_Adjacency(M3, mode='upper', loops='twice') — UPPER collapses TWICE→ONCE; diag stays at 2.0 and 3.0 (un-halved)",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [1.5, 0.0, 2.0], [0.0, 2.5, 3.0]],
            "mode": "upper",
            "loops": "twice",
        },
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 0], [0, 1], [1, 2], [2, 2]],
            "weights": [2.0, 0.5, 2.0, 3.0],
        },
    },
]


ALGO_MANIFESTS: Dict[str, List[Dict[str, Any]]] = {
    "bfs": BFS_MANIFEST,
    "community_to_membership": COMMUNITY_TO_MEMBERSHIP_MANIFEST,
    "compare_communities": COMPARE_COMMUNITIES_MANIFEST,
    "reindex_membership": REINDEX_MEMBERSHIP_MANIFEST,
    "split_join_distance": SPLIT_JOIN_DISTANCE_MANIFEST,
    "dfs": DFS_MANIFEST,
    "connected_components": CC_MANIFEST,
    "strongly_connected_components": SCC_MANIFEST,
    "distances": DIST_MANIFEST,
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
    "voronoi": VORONOI_MANIFEST,
    "ecc": ECC_PR031_MANIFEST,
    "rich_club_sequence": RICH_CLUB_MANIFEST,
    "community_voronoi": COMMUNITY_VORONOI_MANIFEST,
    "minimum_spanning_tree": SPANNING_TREE_MANIFEST,
    "max_flow_value": MAXFLOW_MANIFEST,
    "st_mincut_value": ST_MINCUT_MANIFEST,
    "st_edge_connectivity": ST_EDGE_CONN_MANIFEST,
    "edge_disjoint_paths": ED_PATHS_MANIFEST,
    "st_vertex_connectivity": ST_VCONN_MANIFEST,
    "vertex_disjoint_paths": VDP_MANIFEST,
    "vertex_connectivity": VCONN_GLOBAL_MANIFEST,
    "edge_connectivity": ECONN_GLOBAL_MANIFEST,
    "mincut_value": MINCUT_VALUE_MANIFEST,
    "st_mincut": ST_MINCUT_PARTITION_MANIFEST,
    "gomory_hu_tree": GOMORY_HU_MANIFEST,
    "dominator_tree": DOMINATOR_TREE_MANIFEST,
    "erdos_renyi_gnp": ERDOS_RENYI_GNP_MANIFEST,
    "erdos_renyi_gnm": ERDOS_RENYI_GNM_MANIFEST,
    "barabasi_game_bag": BARABASI_BAG_MANIFEST,
    "growing_random_game": GROWING_RANDOM_MANIFEST,
    "tree_game_lerw": TREE_LERW_MANIFEST,
    "grg_game": GRG_MANIFEST,
    "forest_fire_game": FOREST_FIRE_MANIFEST,
    "preference_game": PREFERENCE_MANIFEST,
    "asymmetric_preference_game": ASYMMETRIC_PREFERENCE_MANIFEST,
    "establishment_game": ESTABLISHMENT_MANIFEST,
    "callaway_traits_game": CALLAWAY_TRAITS_MANIFEST,
    "cited_type_game": CITED_TYPE_MANIFEST,
    "citing_cited_type_game": CITING_CITED_TYPE_MANIFEST,
    "lastcit_game": LASTCIT_MANIFEST,
    "recent_degree_game": RECENT_DEGREE_MANIFEST,
    "barabasi_game_psumtree": BARABASI_PSUMTREE_MANIFEST,
    "barabasi_aging_game": BARABASI_AGING_MANIFEST,
    "dot_product_game": DOT_PRODUCT_MANIFEST,
    "correlated_game": CORRELATED_MANIFEST,
    "correlated_pair_game": CORRELATED_PAIR_MANIFEST,
    "degree_sequence_game_configuration": DEGREE_SEQUENCE_CONFIG_MANIFEST,
    "degree_sequence_game_fast_heur_simple": DEGREE_SEQUENCE_FAST_HEUR_MANIFEST,
    "degree_sequence_game_configuration_simple": DEGREE_SEQUENCE_CONFIG_SIMPLE_MANIFEST,
    "degree_sequence_game_edge_switching_simple": DEGREE_SEQUENCE_EDGE_SWITCHING_SIMPLE_MANIFEST,
    "degree_sequence_game_vl": DEGREE_SEQUENCE_VL_MANIFEST,
    "simple_interconnected_islands_game": ISLANDS_MANIFEST,
    "k_regular_game": K_REGULAR_MANIFEST,
    "watts_strogatz_game": WATTS_STROGATZ_MANIFEST,
    "sbm_game": SBM_MANIFEST,
    "hsbm_game": HSBM_MANIFEST,
    "hsbm_list_game": HSBM_LIST_MANIFEST,
    "chung_lu_game": CHUNG_LU_MANIFEST,
    "static_fitness_game": STATIC_FITNESS_MANIFEST,
    "static_power_law_game": STATIC_POWER_LAW_MANIFEST,
    "ring_graph": RING_MANIFEST,
    "star_graph": STAR_MANIFEST,
    "wheel_graph": WHEEL_MANIFEST,
    "kary_tree": KARY_TREE_MANIFEST,
    "symmetric_tree": SYMMETRIC_TREE_MANIFEST,
    "regular_tree": REGULAR_TREE_MANIFEST,
    "hypercube": HYPERCUBE_MANIFEST,
    "hamming": HAMMING_MANIFEST,
    "square_lattice": SQUARE_LATTICE_MANIFEST,
    "generalized_petersen": GENERALIZED_PETERSEN_MANIFEST,
    "circulant": CIRCULANT_MANIFEST,
    "de_bruijn": DE_BRUIJN_MANIFEST,
    "kautz": KAUTZ_MANIFEST,
    "full_graph": FULL_MANIFEST,
    "full_citation": FULL_CITATION_MANIFEST,
    "full_multipartite": FULL_MULTIPARTITE_MANIFEST,
    "linegraph": LINEGRAPH_MANIFEST,
    "from_prufer": PRUFER_MANIFEST,
    "tree_from_parent_vector": TREE_FROM_PARENT_VECTOR_MANIFEST,
    "lcf": LCF_MANIFEST,
    "mycielski_graph": MYCIELSKI_GRAPH_MANIFEST,
    "mycielskian": MYCIELSKIAN_MANIFEST,
    "famous": FAMOUS_MANIFEST,
    "atlas": ATLAS_MANIFEST,
    "create": CREATE_MANIFEST,
    "triangular_lattice": TRIANGULAR_LATTICE_MANIFEST,
    "hexagonal_lattice": HEXAGONAL_LATTICE_MANIFEST,
    "adjacency": ADJACENCY_MANIFEST,
    "weighted_adjacency": WEIGHTED_ADJACENCY_MANIFEST,
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
        # `community_to_membership` is a dendrogram helper, not a
        # graph algorithm — bypass the graph_factory flow.
        if algo == "community_to_membership":
            nodes = int(entry["nodes"])
            payload = {
                "source": "py",
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
                "source": "py",
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
                "source": "py",
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
                "source": "py",
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
            "preference_game",
            "asymmetric_preference_game",
            "establishment_game",
            "callaway_traits_game",
            "cited_type_game",
            "citing_cited_type_game",
            "lastcit_game",
            "recent_degree_game",
            "barabasi_game_psumtree",
            "barabasi_aging_game",
            "dot_product_game",
            "correlated_pair_game",
            "degree_sequence_game_configuration",
            "degree_sequence_game_fast_heur_simple",
            "degree_sequence_game_configuration_simple",
            "degree_sequence_game_edge_switching_simple",
            "degree_sequence_game_vl",
            "simple_interconnected_islands_game",
            "k_regular_game",
            "watts_strogatz_game",
            "sbm_game",
            "hsbm_game",
            "hsbm_list_game",
            "chung_lu_game",
            "static_fitness_game",
            "static_power_law_game",
            "ring_graph",
            "star_graph",
            "wheel_graph",
            "kary_tree",
            "symmetric_tree",
            "regular_tree",
            "hypercube",
            "hamming",
            "square_lattice",
            "generalized_petersen",
            "circulant",
            "de_bruijn",
            "kautz",
            "full_graph",
            "full_citation",
            "full_multipartite",
            "from_prufer",
            "tree_from_parent_vector",
            "lcf",
            "mycielski_graph",
            "famous",
            "atlas",
            "create",
            "triangular_lattice",
            "hexagonal_lattice",
            "adjacency",
            "weighted_adjacency",
        ):
            # Generators produce a graph from params alone; the
            # graph payload is a placeholder. The expected block carries
            # structural invariants (vcount/ecount/directed and, for BA,
            # the `ba_temporal_order` flag).
            payload = {
                "source": "py",
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
        elif algo == "voronoi":
            g = entry["graph_factory"]()
            graph_payload = graph_to_payload(g)
            weights = entry.get("graph_weights")
            if weights is not None:
                graph_payload["weights"] = list(weights)
            generators = [int(v) for v in entry["params"]["generators"]]
            mode = str(entry["params"]["mode"]).lower()
            tiebreaker = str(entry["params"]["tiebreaker"]).lower()
            ig_mode = {
                "out": "out",
                "in": "in",
                "all": "all",
            }[mode]
            # Per-generator distance row from every vertex to that
            # generator (mode-aware). Then min-merge under the chosen
            # tiebreaker — exactly mirrors voronoi.c lines 30-309 with
            # the simplification that we do not need the mindist-aware
            # early subtree pruning since we are filling a static fixture.
            dist_matrix = []
            for gen in generators:
                # `Graph.distances(source, target, mode)` returns a list
                # of lists [[source -> v] for v in target]. With a single
                # source the outer list has length 1.
                row = g.distances(source=gen, mode=ig_mode, weights=weights)[0]
                dist_matrix.append(row)
            n = g.vcount()
            membership: List[Any] = [None] * n
            distances: List[Any] = [None] * n
            for v in range(n):
                best = None
                best_idx: Any = None
                for i in range(len(generators)):
                    d = dist_matrix[i][v]
                    if d == float("inf"):
                        continue
                    if best is None or d < best:
                        best = d
                        best_idx = i
                    elif d == best and tiebreaker == "last":
                        best_idx = i
                if best is None:
                    membership[v] = None
                    distances[v] = None
                else:
                    membership[v] = int(best_idx)
                    distances[v] = float(best)
            payload = {
                "source": "py",
                "origin": entry["origin"],
                "graph": graph_payload,
                "algo": algo,
                "params": {
                    "generators": generators,
                    "mode": mode,
                    "tiebreaker": tiebreaker,
                },
                "expected": {
                    "membership": membership,
                    "distances": distances,
                },
            }
        else:
            g: ig.Graph = entry["graph_factory"]()
            graph_payload = graph_to_payload(g)
            if "graph_weights" in entry:
                graph_payload["weights"] = list(entry["graph_weights"])
            payload = {
                "source": "py",
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
