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
    "community_voronoi": COMMUNITY_VORONOI_MANIFEST,
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
