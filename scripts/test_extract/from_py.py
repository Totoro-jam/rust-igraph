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

ALGO_MANIFESTS: Dict[str, List[Dict[str, Any]]] = {
    "bfs": BFS_MANIFEST,
    "dfs": DFS_MANIFEST,
    "connected_components": CC_MANIFEST,
    "strongly_connected_components": SCC_MANIFEST,
    "distances": DIST_MANIFEST,
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
    "disjoint_union_many": DU_MANY_MANIFEST,
    "coreness_with_mode": CORENESS_MODE_MANIFEST,
    "assortativity_degree_directed": ASSORT_DIR_MANIFEST,
    "complementer": COMPLEMENTER_MANIFEST,
    "closeness_weighted": CLOSENESS_W_MANIFEST,
    "harmonic_centrality_weighted": HARMONIC_W_MANIFEST,
    "betweenness_weighted": BETW_W_MANIFEST,
    "edge_betweenness_weighted": EDGE_BETW_W_MANIFEST,
    "pagerank_weighted": PAGERANK_W_MANIFEST,
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
