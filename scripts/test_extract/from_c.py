#!/usr/bin/env python3
"""Extract conformance fixtures from `references/igraph/tests/unit/`.

Phase 0 demo: ships a hand-curated manifest covering BFS so the conformance
flow is exercised end to end. As more AWUs land the manifest grows; later
phases (BOOT-29 in Phase 1) replace this with a proper C-token + `.out`
parser that scales to all 425 C tests.

Output: `tests/conformance/c/<algo>/<case>.json`

Usage:
    .venv/bin/python -m scripts.test_extract.from_c --algo bfs
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Callable, Dict, List

import igraph as ig

REPO_ROOT = Path(__file__).resolve().parents[2]
C_TESTS_DIR = REPO_ROOT / "references/igraph/tests/unit"
OUT_DIR = REPO_ROOT / "tests/conformance/c"


def _ring(n: int, circular: bool = True) -> ig.Graph:
    """Wrap igraph's Ring; `circular=False` produces a path (matches C's circular=0)."""
    return ig.Graph.Ring(n=n, directed=False, mutual=False, circular=circular)


def _kary_tree(n: int, k: int) -> ig.Graph:
    return ig.Graph.Tree(n=n, children=k, mode="undirected")


# Manifest of known C tests we mirror. Each entry is independent and produces
# one JSON fixture. Adding a row: pick a C test, encode graph + algo + expected
# output, drop the first ≤N tokens of `.out` for the expected.
BFS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "path10_root0",
        # NOTE: bfs_simple.c calls igraph_ring(g, 10, UNDIRECTED, mutual=0, circular=0)
        # — the 5th arg `circular=0` makes it a *path*, not a closed ring.
        "origin": "bfs_simple.c:igraph_ring(10, circular=0) BFS root=0  [== path of 10]",
        "graph_factory": lambda: _ring(10, circular=False),
        "algo": "bfs",
        "params": {"root": 0},
        # First line of bfs_simple.out: "( 0 1 2 3 4 5 6 7 8 9 )"
        "expected": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    },
    {
        "case": "kary_tree20_k2_root0",
        "origin": "bfs_simple.c:kary_tree(20,2) BFS root=0",
        "graph_factory": lambda: _kary_tree(20, 2),
        "algo": "bfs",
        "params": {"root": 0},
        # Second BFS in bfs_simple.out:
        # "( 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 )"
        "expected": list(range(20)),
    },
]

DFS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "kary_tree20_k2_root0",
        # No dedicated dfs.c / dfs.out exists in tests/unit/ upstream.
        # We reuse the kary_tree(20, 2) graph from bfs_simple.c and
        # compute the expected DFS pre-order via python-igraph (which
        # is a thin Cython wrapper over the same igraph_dfs() in
        # references/igraph/src/graph/visitors.c). That makes this
        # fixture C-equivalent in lineage even though no .out file is
        # the source of truth.
        "origin": "bfs_simple.c:kary_tree(20,2) graph; DFS expected via python-igraph 0.11.x = igraph C",
        "graph_factory": lambda: _kary_tree(20, 2),
        "algo": "dfs",
        "params": {"root": 0},
        "expected": [0, 2, 6, 14, 13, 5, 12, 11, 1, 4, 10, 9, 19, 3, 8, 18, 17, 7, 16, 15],
    },
]

CC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "components_two_isolated_vertices",
        # components.c tests singleton + null + connected; we mirror the
        # "n vertices, no edges" case which gives n components, ids 0..n-1.
        "origin": "components.c:'Singleton graph (connected)' adapted; 2 isolated vertices",
        "graph_factory": lambda: ig.Graph(n=2, edges=[], directed=False),
        "algo": "connected_components",
        "params": {},
        "expected": {"membership": [0, 1], "count": 2},
    },
    {
        "case": "components_path5_one_component",
        "origin": "components.c-style: a 5-vertex path is one weakly-connected component",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "connected_components",
        "params": {},
        "expected": {"membership": [0, 0, 0, 0, 0], "count": 1},
    },
]

TRI_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_triangles_c_K4",
        # global_transitivity.c lines 51-56 use Full(3) → 1 triangle.
        # We use K4 (Full(4)) for a slightly less degenerate fixture
        # → 4 triangles.
        "origin": "global_transitivity.c-style: K4 has 4 triangles",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "count_triangles",
        "params": {},
        "expected": 4,
    },
]

KNN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knn_c_path_5",
        # Path 0-1-2-3-4: knn = [2, 1.5, 2, 1.5, 2] (verified above).
        "origin": "constructed: 5-path knn vector",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "avg_nearest_neighbor_degree",
        "params": {},
        "expected": [2.0, 1.5, 2.0, 1.5, 2.0],
    },
]

KNN_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knn_weighted_c_triangle_unequal",
        # Triangle 0-1-2 with weights e0=(0,1)=1, e1=(1,2)=2, e2=(2,0)=4.
        # All degrees = 2. Weighted knn for any vertex must equal 2.0
        # because every neighbour has the same degree (2) — the weight
        # cancels out (Σ w·k / Σ w = k for constant k).
        "origin": "constructed: triangle with non-uniform weights, all-deg-2 → knn=2",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 4.0],
        "algo": "avg_nearest_neighbor_degree_weighted",
        "params": {},
        "expected": [2.0, 2.0, 2.0],
    },
]

KNNK_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knnk_c_star_4",
        # Star K_{1,3}: degrees [3, 1, 1, 1]. knn = [1, 3, 3, 3].
        # knnk[0] (deg 1) = avg(3, 3, 3) = 3.0.
        # knnk[1] (deg 2) = NaN → None.
        # knnk[2] (deg 3) = 1.0.
        "origin": "constructed: 4-star, hand-checked knnk (deg 1→3, deg 2→None, deg 3→1)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "algo": "knnk",
        "params": {},
        "expected": [3.0, None, 1.0],
    },
]

KNNK_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "knnk_weighted_c_triangle_unequal",
        # Triangle 0-1-2 with weights (1, 2, 4): all degrees=2, all knn=2.
        # knnk_weighted[1] (deg 2) = pooled (Σ sum / Σ str). Each vertex
        # has sum/str = 2.0; pooled also = 2.0.
        "origin": "constructed: weighted triangle, all-deg-2 → knnk[deg=2] = 2.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 4.0],
        "algo": "knnk_weighted",
        "params": {},
        # knnk has length max_deg = 2. Bucket 0 (deg 1) is None (no deg-1
        # vertices); bucket 1 (deg 2) is 2.0.
        "expected": [None, 2.0],
    },
]

RECIP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reciprocity_c_directed_3_cycle_zero",
        "origin": "directed 3-cycle has no reciprocal edges → 0.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "reciprocity",
        "params": {},
        "expected": 0.0,
    },
]

EIGEN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eigenvector_c_triangle",
        # K3: every vertex has identical eigenvector centrality 1.0.
        "origin": "constructed: triangle; uniform eigenvector centrality 1.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "eigenvector_centrality",
        "params": {},
        "expected": [1.0, 1.0, 1.0],
    },
]

BC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "biconnected_components_c_upstream_fixture",
        # From references/igraph/tests/unit/igraph_biconnected_components.c.
        # 10v graph (vertex 9 isolated), 9 edges. Expected 4 components with
        # APs {2, 5}.
        "origin": "igraph_biconnected_components.c upstream test fixture",
        "graph_factory": lambda: ig.Graph(
            n=10,
            edges=[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (2, 4),
                (4, 5),
                (5, 2),
                (5, 6),
                (7, 8),
            ],
            directed=False,
        ),
        "algo": "biconnected_components",
        "params": {},
        "expected": {
            "count": 4,
            "components": [
                [0, 1, 2, 3],
                [2, 4, 5],
                [5, 6],
                [7, 8],
            ],
            "articulation_points": [2, 5],
        },
    },
]

BC_EDGES_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "biconnected_component_edges_c_two_blocks_plus_pendant",
        # Same upstream fixture used by BC_MANIFEST: 10v graph with 4
        # biconnected components. CC-012 adds the per-component edge
        # set; here we hand-encode the expected partition of edge
        # endpoint pairs (canonicalised as sorted (min, max)).
        "origin": "igraph_biconnected_components.c — component_edges output (CC-012)",
        "graph_factory": lambda: ig.Graph(
            n=10,
            edges=[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (2, 4),
                (4, 5),
                (5, 2),
                (5, 6),
                (7, 8),
            ],
            directed=False,
        ),
        "algo": "biconnected_component_edges",
        "params": {},
        # Per-component edge-pair partition (sorted within each component,
        # outer list sorted lexicographically). Pairs are (min(u,v), max(u,v)).
        # Components: {0,1,2,3} cycle (4 edges), {2,4,5} triangle (3),
        # {5,6} bridge (1), {7,8} bridge (1).
        "expected": sorted(
            [
                sorted([[0, 1], [1, 2], [2, 3], [0, 3]]),
                sorted([[2, 4], [4, 5], [2, 5]]),
                [[5, 6]],
                [[7, 8]],
            ]
        ),
    },
]

PAGERANK_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "pagerank_c_directed_4cycle",
        # Directed 4-cycle: every vertex has identical PageRank = 0.25.
        "origin": "constructed: directed 4-cycle; uniform PageRank 0.25",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=True
        ),
        "algo": "pagerank",
        "params": {},
        "expected": [0.25, 0.25, 0.25, 0.25],
    },
]

EDGE_BETW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_betweenness_c_path4",
        # Path 0-1-2-3 (3 edges): textbook edge betweenness 3, 4, 3.
        "origin": "constructed: 4-path; edge_betweenness via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "edge_betweenness",
        "params": {},
        "expected": [3.0, 4.0, 3.0],
    },
]

BETW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "betweenness_c_path5",
        # Path 0-1-2-3-4: textbook Brandes result.
        "origin": "constructed: 5-path; betweenness via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "betweenness",
        "params": {},
        "expected": [0.0, 3.0, 4.0, 3.0, 0.0],
    },
]

HARMONIC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "harmonic_c_path5",
        # Path 0-1-2-3-4 — symmetric profile peaking at the centre.
        "origin": "constructed: 5-path; harmonic centrality via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "harmonic_centrality",
        "params": {},
        "expected": [
            0.5208333333333333,
            0.7083333333333334,
            0.75,
            0.7083333333333334,
            0.5208333333333333,
        ],
    },
]

CLOSE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "closeness_c_path5",
        # Path 0-1-2-3-4: ends 0.4, near-ends 4/7, centre 4/6.
        "origin": "constructed: 5-path; closeness via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "closeness",
        "params": {},
        "expected": [0.4, 4.0 / 7.0, 4.0 / 6.0, 4.0 / 7.0, 0.4],
    },
]

ASSORT_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assort_w_c_path_3_non_uniform",
        # Path 0-1-2 weights (1, 4): strengths [1, 5, 4].
        # By the weighted Pearson formula:
        #   W = 5; num1=85/5=17; num2=42/10=4.2 → ^2 = 17.64; den1=190/10=19
        #   r = (17 - 17.64) / (19 - 17.64) = -0.64 / 1.36
        # Hand-computed; non-unit weights so python-igraph can't oracle this.
        "origin": "constructed: 3-path weights (1, 4); hand-computed -0.64/1.36",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0],
        "algo": "assortativity_degree_weighted",
        "params": {},
        "expected": -0.64 / 1.36,
    },
]

PAGERANK_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "pagerank_w_c_directed_4cycle_unit_weights",
        # Unit-weight directed 4-cycle: PageRank is uniform 0.25 by
        # symmetry — same as PR-011 fixture with weights present.
        "origin": "constructed: directed 4-cycle, unit weights → uniform 0.25",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=True
        ),
        "graph_weights": [1.0, 1.0, 1.0, 1.0],
        "algo": "pagerank_weighted",
        "params": {},
        "expected": [0.25, 0.25, 0.25, 0.25],
    },
]

EDGE_BETW_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_betw_w_c_path_4_unit_weights",
        # 4-path with unit weights collapses to PR-010's [3, 4, 3].
        "origin": "constructed: 4-path with unit weights matches PR-010",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0],
        "algo": "edge_betweenness_weighted",
        "params": {},
        "expected": {
            "edges": [[0, 1], [1, 2], [2, 3]],
            "values": [3.0, 4.0, 3.0],
        },
    },
]

BETW_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "betw_w_c_path_swaps_via_higher_weight",
        # Triangle with weight (1, 1, 5) on (0,1)/(1,2)/(0,2): direct
        # (0,2) has cost 5, but 0→1→2 has cost 2 → vertex 1 is the
        # only intermediary. Brandes raw count for the unordered
        # pair (0,2) is 2 (counted once from each direction);
        # undirected halves to 1.0.
        "origin": "constructed: triangle with weights (1,1,5) routes through vertex 1",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 5.0],
        "algo": "betweenness_weighted",
        "params": {},
        "expected": [0.0, 1.0, 0.0],
    },
]

HARMONIC_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "harmonic_w_c_path_w1_w2",
        # 3-path with weights (1, 2). Distances from 0: {1@1, 2@3} →
        # 1+1/3 = 4/3 / 2 = 2/3.
        "origin": "constructed: 3-path with weights (1, 2); centre and ends",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 2.0],
        "algo": "harmonic_centrality_weighted",
        "params": {},
        "expected": [
            2.0 / 3.0,
            0.75,
            5.0 / 12.0,
        ],
    },
]

CLOSENESS_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "closeness_weighted_c_star_non_uniform",
        # 4-star with non-uniform weights; centre's closeness =
        # 3 / (1+2+3) = 0.5.
        "origin": "constructed: 4-star with non-uniform weights (1,2,3); "
        "centre closeness = 0.5",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 3.0],
        "algo": "closeness_weighted",
        "params": {},
        "expected": [0.5, 3.0 / 8.0, 0.3, 0.25],
    },
]

COMPLEMENTER_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "complementer_c_path_three_no_loops",
        # From references/igraph/examples/simple/igraph_complementer.c style:
        # 3-path complementer (no loops) is the single missing chord (0,2).
        "origin": "constructed: 3-path; complementer (no loops) = single chord (0,2)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "complementer",
        "params": {"loops": False},
        "expected": {
            "vcount": 3,
            "directed": False,
            "edges": [[0, 2]],
        },
    },
]

DIJKSTRA_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_c_triangle_with_shortcut",
        # Standard "shortcut" case from references/igraph/examples/simple/
        # igraph_distances_dijkstra.c style: triangle with weights 1, 4, 2
        # so the path 0->1->2 (cost 3) is shorter than the direct 0-2
        # edge (cost 4).
        "origin": "constructed: triangle (1,4,2) with shortcut via vertex 1",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "dijkstra_distances",
        "params": {"source": 0},
        "expected": [0.0, 1.0, 3.0],
    },
]

MODULARITY_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_directed_c_two_triangles_bridge",
        # Mirrors C unit test: two directed triangles + bridge, with
        # partition {0,1,2}/{3,4,5}. Hand-checked Q = 18/49 ≈ 0.367.
        "origin": "constructed: two directed triangles + bridge 2→3",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)],
            directed=True,
        ),
        "algo": "modularity_directed",
        "params": {"membership": [0, 0, 0, 1, 1, 1], "resolution": 1.0},
        "expected": 18.0 / 49.0,
    },
]

ASSORT_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_degree_directed_c_chain_with_branch",
        # Mirrors C unit test: 0→1, 1→2, 0→2.
        # Out-deg [2,1,0], in-deg [0,1,2]; Pearson r = -0.5 (variance
        # well-defined on both sides).
        "origin": "constructed: 0→1, 1→2, 0→2 (chain with branch)",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (0, 2)],
            directed=True,
        ),
        "algo": "assortativity_degree_directed",
        "params": {},
        "expected": -0.5,
    },
]

CORENESS_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "coreness_with_mode_c_directed_complete_3_out",
        # Mirrors C unit test: directed K3 (each pair has both
        # directions). Out-degrees are all 2 → out-cores all 2.
        "origin": "constructed: directed K3 (mutual on all pairs), out-mode",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 0), (1, 2), (2, 1), (0, 2), (2, 0)],
            directed=True,
        ),
        "algo": "coreness_with_mode",
        "params": {"mode": "out"},
        "expected": [2, 2, 2],
    },
]

DU_MANY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "disjoint_union_many_c_three_triangles",
        # Mirrors the C unit test for igraph_disjoint_union_many: three
        # triangles → 9-vertex 9-edge graph with three disjoint K3s.
        "origin": "constructed: three K3 triangles",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (2, 0)],
            directed=False,
        ),
        "algo": "disjoint_union_many",
        "params": {
            "extra_graphs": [
                {
                    "n": 3,
                    "edges": [[0, 1], [1, 2], [2, 0]],
                    "directed": False,
                    "weights": None,
                },
                {
                    "n": 3,
                    "edges": [[0, 1], [1, 2], [2, 0]],
                    "directed": False,
                    "weights": None,
                },
            ]
        },
        "expected": {
            "vcount": 9,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [1, 2],
                [3, 4], [3, 5], [4, 5],
                [6, 7], [6, 8], [7, 8],
            ],
        },
    },
]

IS_SIMPLE_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_simple_with_mode_c_directed_mutual_pair_undirected_view",
        # Mirrors the test_is_simple unit test: a directed mutual pair
        # is simple structurally but NOT simple as undirected
        # (collapses to a doubled undirected edge).
        "origin": "constructed: directed mutual pair, undirected view",
        "graph_factory": lambda: ig.Graph(
            n=2,
            edges=[(0, 1), (1, 0)],
            directed=True,
        ),
        "algo": "is_simple_with_mode",
        "params": {"directed_as_undirected": True},
        "expected": False,
    },
]

MODULARITY_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_weighted_c_balanced_heavy_internal",
        # Mirrors the test_modularity_weighted style fixture: K3 ∪ K3
        # + bridge with internal weight 10× and bridge 0.1×, partition
        # [0,0,0,1,1,1]. Python-igraph cross-validated.
        "origin": "constructed: K3 ∪ K3 + bridge, balanced heavy internal weights",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
            directed=False,
        ),
        "graph_weights": [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 0.1],
        "algo": "modularity_weighted",
        "params": {"membership": [0, 0, 0, 1, 1, 1], "resolution": 1.0},
        # Hand-checked: W = 60.1, w_internal = 120, e_norm = 120/120.2,
        # s[c0] = s[c1] = 60.1/120.2 = 0.5 each.
        # Q ≈ 120/120.2 - 2*0.25 ≈ 0.99834 - 0.5 ≈ 0.49834.
        "expected": 0.4983361064891847,
    },
]

RECIP_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reciprocity_with_mode_c_ratio_partial",
        # Mirrors igraph C unit test for IGRAPH_RECIPROCITY_RATIO:
        # 0→1, 1→0 (mutual), 0→2 (one-way) → rec=2, nonrec=2,
        # ratio = 2/4 = 0.5 (different from Default's 2/3).
        "origin": "constructed: mutual 0↔1 + one-way 0→2, ratio mode",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 0), (0, 2)],
            directed=True,
        ),
        "algo": "reciprocity_with_mode",
        "params": {"ignore_loops": False, "mode": "ratio"},
        "expected": 0.5,
    },
]

CORENESS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "coreness_c_triangle_with_pendant",
        # Mirrors references/igraph/examples/simple/igraph_coreness.c
        # style: triangle 0-1-2 with a pendant vertex 3 attached to 2.
        # Pendant has degree 1 so it is in the 1-core; once peeled the
        # triangle survives so 0, 1, 2 sit in the 2-core.
        "origin": "constructed: triangle 0-1-2 + pendant 3 attached to 2",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (0, 2), (2, 3)],
            directed=False,
        ),
        "algo": "coreness",
        "params": {},
        "expected": [2, 2, 2, 1],
    },
]

FW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "floyd_warshall_c_directed_chain_with_shortcut",
        # Mirrors the "Weighted directed" case from
        # references/igraph/tests/unit/igraph_distances_floyd_warshall.c
        # in spirit but on a smaller, hand-checkable graph: a directed
        # 4-vertex chain 0→1→2→3 with a costly direct shortcut 0→3 (5).
        # The chain (cost 1+1+1=3) wins over the direct edge.
        "origin": "constructed: directed chain 0->1->2->3 + shortcut 0->3@5",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (0, 3)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "floyd_warshall_distances",
        "params": {},
        # Out-mode FW on directed: row[i] holds dist(i, *).
        # 0 reaches everyone via the chain; 1, 2, 3 only reach
        # forward.
        "expected": [
            [0.0, 1.0, 2.0, 3.0],
            [None, 0.0, 1.0, 2.0],
            [None, None, 0.0, 1.0],
            [None, None, None, 0.0],
        ],
    },
]

DISJOINT_UNION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "disjoint_union_c_two_triangles",
        # From references/igraph/examples/simple/igraph_disjoint_union.c style:
        # disjoint union of two triangles → 6 vertices, 6 edges, two
        # disjoint K3 components.
        "origin": "constructed: disjoint union of two K3 triangles",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "disjoint_union",
        "params": {
            "right_graph": {
                "n": 3,
                "edges": [[0, 1], [1, 2], [2, 0]],
                "directed": False,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 6,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 2], [3, 4], [3, 5], [4, 5]],
        },
    },
]

IS_LOOP_PER_EDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_loop_c_mixed_self_loops",
        # From references/igraph/examples/simple/igraph_is_loop.c style.
        "origin": "constructed: 3 edges with one self-loop in the middle",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (2, 2), (1, 2)], directed=False
        ),
        "algo": "is_loop",
        "params": {},
        # Per-edge result depends on the wire-format edge order. After
        # round-trip the edges become [(0,1),(1,2),(2,2)] so the loop
        # mask is [F, F, T]. Conformance compares as a multiset though,
        # so we record the canonical sorted form: one True, two False.
        "expected": [False, False, True],
    },
]

IS_MULTIPLE_PER_EDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_multiple_c_two_parallels",
        # Two parallel edges + one normal: only the second-or-more
        # appearance is True. After wire round-trip edges become
        # [(0,1),(0,1),(1,2)]; canonical first→False, dup→True, lone→False.
        # Conformance compares sorted multisets, so we record [F, F, T].
        "origin": "constructed: two parallel (0,1) + one (1,2); two False / one True",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 1), (1, 2)], directed=False
        ),
        "algo": "is_multiple",
        "params": {},
        "expected": [False, False, True],
    },
]

HAS_LOOP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "has_loop_c_self_loop_present",
        # From references/igraph/examples/simple/igraph_is_loop.c style:
        # tiny graph with one self-loop should report has_loop=true.
        "origin": "constructed: graph with one self-loop; has_loop=true",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 0), (1, 2)], directed=False
        ),
        "algo": "has_loop",
        "params": {},
        "expected": True,
    },
]

HAS_MULTIPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "has_multiple_c_two_parallel_edges",
        "origin": "constructed: two parallel undirected edges; has_multiple=true",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 1), (1, 2)], directed=False
        ),
        "algo": "has_multiple",
        "params": {},
        "expected": True,
    },
]

IS_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_simple_c_path_is_simple",
        # Plain undirected path → simple (no loops, no parallels).
        "origin": "constructed: undirected 4-path; simple",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "is_simple",
        "params": {},
        "expected": True,
    },
]

MODULARITY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "modularity_c_two_triangles_bridge_split",
        # Two K3 triangles connected by a bridge edge; partition {0,1,2}
        # vs {3,4,5} → Q = 6/7 - 0.5 = 0.357142857... (exact rational
        # representable as f64). This is the canonical hand-computable
        # case used in the unit tests; matches python-igraph + rigraph.
        "origin": "constructed: two K3 + bridge; partition {0,1,2}/{3,4,5}; "
        "Q = 6/7 - 0.5",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5), (2, 3)],
            directed=False,
        ),
        "algo": "modularity",
        "params": {"membership": [0, 0, 0, 1, 1, 1], "resolution": 1.0},
        "expected": 6.0 / 7.0 - 0.5,
    },
]

SIMPLIFY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "simplify_c_undirected_loops_and_multi",
        # From references/igraph/examples/simple/igraph_simplify.c case 2:
        # undirected (1,0)(0,1)(1,0)(0,1)(0,1) → simplify(true,true) leaves
        # exactly 1 edge (a single 0-1 edge).
        "origin": "igraph_simplify.c case 2: undirected 5x parallel 0-1; simplify all",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(1, 0), (0, 1), (1, 0), (0, 1), (0, 1)], directed=False
        ),
        "algo": "simplify",
        "params": {"remove_multiple": True, "remove_loops": True},
        "expected": {"vcount": 2, "directed": False, "edges": [[0, 1]]},
    },
]

TC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitive_closure_c_directed_path3",
        # Directed 0->1->2: closure adds 0->2 → 3 edges.
        "origin": "constructed: directed path 0->1->2; transitive closure adds 0->2",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "algo": "transitive_closure",
        "params": {},
        "expected": {
            "vcount": 3,
            "directed": True,
            "edges": [[0, 1], [0, 2], [1, 2]],
        },
    },
]

REACH_MATRIX_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "reachability_matrix_c_directed_3cycle",
        # Directed 3-cycle: every vertex reaches every other (full True matrix).
        "origin": "constructed: directed 3-cycle 0->1->2->0; full True matrix",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "reachability_matrix",
        "params": {},
        "expected": [[True, True, True], [True, True, True], [True, True, True]],
    },
]

REACH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_reachable_c_directed_chain",
        # Directed chain 0->1->2->3: counts = [4, 3, 2, 1].
        "origin": "constructed: directed chain 0->1->2->3; per-vertex reachable counts",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "algo": "count_reachable",
        "params": {},
        "expected": [4, 3, 2, 1],
    },
]

EUL_PATH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eulerian_path_c_triangle_walk_len_3",
        # Triangle has Eulerian cycle; the walk must have length 3.
        # We compare only `len(walk)` since multiple valid walks exist —
        # encode the expected length and let the conformance test runner
        # fall back to length-only check via a bespoke runner.
        "origin": "constructed: triangle — Eulerian cycle exists, walk has length 3",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "eulerian_path",
        "params": {},
        "expected": 3,
    },
    {
        "case": "eulerian_path_c_directed_3_cycle_walk_len_3",
        # Directed 3-cycle 0->1->2->0: Eulerian cycle exists, walk len 3.
        "origin": "directed 3-cycle — Eulerian cycle, walk has length 3",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "eulerian_path",
        "params": {},
        "expected": 3,
    },
]

ASSORT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_c_zachary",
        # Famous("Zachary") is the canonical example for degree
        # assortativity (negative — high-deg core attached to low-deg leaves).
        "origin": "Famous('Zachary'); assortativity_degree via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "assortativity_degree",
        "params": {},
        "expected": -0.47561309768461435,
    },
]

DENSITY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "density_c_zachary",
        "origin": "Famous('Zachary'); density via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "density",
        "params": {},
        "expected": 0.13903743315508021,
    },
]

MEANDIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mean_distance_c_zachary",
        "origin": "Famous('Zachary'); avg path length via python-igraph 0.11",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "mean_distance",
        "params": {},
        "expected": 2.408199643493761,
    },
]

LTRANS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_local_c_K4",
        # K4: every vertex has clustering 1.0.
        "origin": "K4 — every vertex has 2 neighbours that are adjacent → 1.0",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "transitivity_local_undirected",
        "params": {},
        "expected": [1.0, 1.0, 1.0, 1.0],
    },
]

TRANS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_undirected_c_zachary",
        # global_transitivity.c line 65-69: famous("Zachary").
        # Expected from .out line 11: 0.255682 (printed at %g precision;
        # actual = 0.25568181818181815). Use the high-precision value.
        "origin": "global_transitivity.c:line 67 — Famous('Zachary') transitivity",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "transitivity_undirected",
        "params": {},
        "expected": 0.2556818181818182,
    },
    {
        "case": "transitivity_undirected_c_K4",
        # global_transitivity.c line 51-56: Full(3) → 1.
        # K4 also has full transitivity 1.0.
        "origin": "global_transitivity.c-style: K4 transitivity = 1.0",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "transitivity_undirected",
        "params": {},
        "expected": 1.0,
    },
]

DIAM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_c_ring10_directed",
        # igraph_diameter.c lines 46-49: directed ring(10) with no mutual
        # arcs has diameter 9 (longest geodesic 0 → 9 via 9 edges).
        "origin": "igraph_diameter.c:lines 46-49 — directed ring(10) diameter 9",
        "graph_factory": lambda: ig.Graph.Ring(
            n=10, directed=True, mutual=False, circular=True
        ),
        "algo": "diameter",
        "params": {},
        "expected": 9,
    },
]

ECC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_c_path5",
        # examples/simple/igraph_eccentricity.c uses a hand-built graph.
        # Use a simple 5-path so the expected vector is unambiguous.
        "origin": "constructed: 5-path; expected via python-igraph 0.11 eccentricity()",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "eccentricity",
        "params": {},
        "expected": [4, 3, 2, 3, 4],
    },
]

RAD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_c_path5",
        "origin": "constructed: 5-path; radius = 2 (centre vertex 2)",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "radius",
        "params": {},
        "expected": 2,
    },
]

GIRTH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "girth_c_ring100_with_chord_0_50",
        # examples/simple/igraph_girth.c lines 25-43: ring(100) + chord 0-50
        # → expected girth 51 (chord shortcut creates a 51-cycle).
        "origin": "examples/simple/igraph_girth.c — ring(100) plus chord (0,50); expected girth 51",
        "graph_factory": lambda: ig.Graph.Ring(
            n=100, directed=False, mutual=False, circular=True
        )
        + [(0, 50)],
        "algo": "girth",
        "params": {},
        "expected": 51,
    },
    {
        "case": "girth_c_null_graph_infinity",
        # examples/simple/igraph_girth.c lines 45-57: ring(0) → IGRAPH_INFINITY.
        # We encode that as null in JSON / None in Rust.
        "origin": "examples/simple/igraph_girth.c — null graph yields IGRAPH_INFINITY",
        "graph_factory": lambda: ig.Graph(n=0, edges=[], directed=False),
        "algo": "girth",
        "params": {},
        "expected": None,
    },
]

ISBI_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_biconnected_c_two_triangles_share_vertex_2",
        # igraph_is_biconnected.c lines 50-57: 6 vertices, two triangles
        # 0-1-2-3-0 (4-cycle) and 2-4-5-2 (3-cycle). Vertex 2 is articulation
        # → not biconnected.
        "origin": "igraph_is_biconnected.c:lines 50-57 — 4-cycle and 3-cycle share vertex 2",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 3), (3, 0), (2, 4), (4, 5), (5, 2)],
            directed=False,
        ),
        "algo": "is_biconnected",
        "params": {},
        "expected": False,
    },
    {
        "case": "is_biconnected_c_ring_10",
        # igraph_is_biconnected.c lines 60-63: ring(10) is biconnected.
        "origin": "igraph_is_biconnected.c:lines 60-63 — ring(10) biconnected",
        "graph_factory": lambda: ig.Graph.Ring(
            n=10, directed=False, mutual=False, circular=True
        ),
        "algo": "is_biconnected",
        "params": {},
        "expected": True,
    },
]

BRIDGE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bridges_c_small_2triangles_with_bridge",
        # igraph_bridges.c lines 33-37: 7 vertices, edges
        #   0-1, 1-2, 0-2, 0-3, 3-4, 4-5, 3-5, 4-6
        # → bridges = (3, 7), i.e. edges 0-3 and 4-6.
        "origin": "igraph_bridges.c:lines 33-37 — 7v, two triangles joined by bridge edge 0-3 plus pendant 4-6",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1),
                (1, 2),
                (0, 2),
                (0, 3),
                (3, 4),
                (4, 5),
                (3, 5),
                (4, 6),
            ],
            directed=False,
        ),
        "algo": "bridges",
        "params": {},
        "expected": [3, 7],
    },
    {
        "case": "bridges_c_multiedge_selfloop_keeps_one",
        # igraph_bridges.c lines 47-52: 3v, edges
        #   0-1, 0-1, 1-2, 2-2 → bridge = edge 2 (the unique 1-2).
        "origin": "igraph_bridges.c:lines 47-52 — multi-edges + self-loop, bridge is the unique 1-2",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 1), (1, 2), (2, 2)], directed=False
        ),
        "algo": "bridges",
        "params": {},
        "expected": [2],
    },
]

AP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "biconnected_components_c_fixture",
        # igraph_biconnected_components.c: 10 vertices (extra isolated 9),
        # edges 0-1, 1-2, 2-3, 3-0, 2-4, 4-5, 5-2, 5-6, 7-8.
        # Articulation points per .out: ( 5 2 ) — sorted = [2, 5].
        "origin": "igraph_biconnected_components.c articulation-points sub-test",
        "graph_factory": lambda: ig.Graph(
            n=10,
            edges=[
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (2, 4),
                (4, 5),
                (5, 2),
                (5, 6),
                (7, 8),
            ],
            directed=False,
        ),
        "algo": "articulation_points",
        "params": {},
        # Sorted to match the runner's contract (we sort both sides).
        "expected": [2, 5],
    },
]

EUL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_eulerian_undirected_path_3",
        # igraph_is_eulerian.c line 12: small(IGRAPH_UNDIRECTED, 0,1, 1,2)
        # → "1 0" (line 1 of .out): has_path && !has_cycle.
        "origin": "igraph_is_eulerian.c:line 12 undirected path 0-1-2",
        "graph_factory": lambda: ig.Graph(n=3, edges=[(0, 1), (1, 2)], directed=False),
        "algo": "is_eulerian",
        "params": {},
        "expected": {"has_path": True, "has_cycle": False},
    },
    {
        "case": "is_eulerian_undirected_triangle",
        # igraph_is_eulerian.c line 22: triangle 1,2 / 2,3 / 3,1 — same as
        # vertices 0,1,2 in 0-based world. Expected line 3 of .out: "1 1".
        "origin": "igraph_is_eulerian.c:line 22 undirected triangle (0-based: 0-1-2-0)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "algo": "is_eulerian",
        "params": {},
        "expected": {"has_path": True, "has_cycle": True},
    },
    {
        "case": "is_eulerian_directed_2_disconnected_edges",
        # igraph_is_eulerian.c line 71-ish: undirected (0,1) + (2,3) two
        # disconnected edges → "0 0" (no path, no cycle). Translated to a
        # directed analog via 0->1, 2->3.
        "origin": "igraph_is_eulerian.c:line 71 two disconnected edges (directed adaption)",
        "graph_factory": lambda: ig.Graph(n=4, edges=[(0, 1), (2, 3)], directed=True),
        "algo": "is_eulerian",
        "params": {},
        "expected": {"has_path": False, "has_cycle": False},
    },
]

DIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "kary_tree20_k2_source0",
        # Same graph as bfs_simple.c's kary_tree(20, 2). Expected = BFS
        # distances from vertex 0 (root): 0, then layer-by-layer.
        "origin": "bfs_simple.c:kary_tree(20,2) distances from vertex 0; "
        "expected via python-igraph 0.11 distances() (= igraph C unweighted BFS)",
        "graph_factory": lambda: _kary_tree(20, 2),
        "algo": "distances",
        "params": {"source": 0},
        "expected": [0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4],
    },
]

SCC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "components_c_two_disjoint_3_cycles",
        # components.c:'Two disjoint 3-cycles' (lines 85-92).
        # Upstream prints the membership AFTER `igraph_reindex_membership`
        # (canonicalised to first-seen order). Both python-igraph and our
        # Rust skip that reindex, so the natural Kosaraju order is what
        # we compare against.
        "origin": "components.c:'Two disjoint 3-cycles' two directed 3-cycles SCC, "
        "natural Kosaraju label order (matches python-igraph 0.11)",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)],
            directed=True,
        ),
        "algo": "strongly_connected_components",
        "params": {},
        "expected": {"membership": [1, 1, 1, 0, 0, 0], "count": 2},
    },
    {
        "case": "components_c_directed_2_path",
        # components.c:'Directed 2-path' (lines 76-83). Upstream output
        # `( 0 1 )` is post-reindex; pre-reindex (and python-igraph) it's
        # also `[0, 1]` because each vertex is its own SCC.
        "origin": "components.c:'Directed 2-path' SCC: every vertex its own component",
        "graph_factory": lambda: ig.Graph(n=2, edges=[(0, 1)], directed=True),
        "algo": "strongly_connected_components",
        "params": {},
        "expected": {"membership": [0, 1], "count": 2},
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
    "closeness": CLOSE_MANIFEST,
    "harmonic_centrality": HARMONIC_MANIFEST,
    "betweenness": BETW_MANIFEST,
    "edge_betweenness": EDGE_BETW_MANIFEST,
    "pagerank": PAGERANK_MANIFEST,
    "biconnected_components": BC_MANIFEST,
    "biconnected_component_edges": BC_EDGES_MANIFEST,
    "eigenvector_centrality": EIGEN_MANIFEST,
    "reciprocity": RECIP_MANIFEST,
    "avg_nearest_neighbor_degree": KNN_MANIFEST,
    "avg_nearest_neighbor_degree_weighted": KNN_W_MANIFEST,
    "knnk": KNNK_MANIFEST,
    "knnk_weighted": KNNK_W_MANIFEST,
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
            "source": "c",
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
