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

DECOMPOSE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "decompose_c_two_components",
        # Triangle {0,1,2} ∪ edge {3,4}. BFS-from-actstart visits 0..2
        # then 3..4, all in identity order, so the remapped subgraphs
        # have edges identical to the originals (modulo the 3-4 → 0-1
        # remap in the second component).
        "origin": "constructed: triangle + edge, hand-checked decompose result",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 0), (3, 4)], directed=False
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
                "vcount": 2,
                "directed": False,
                "edges": [[0, 1]],
            },
        ],
    },
]

TRANS_BARRAT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "transitivity_barrat_c_triangle_unequal",
        # Triangle 0-1-2 with edges e0=(0,1)=1, e1=(1,2)=2, e2=(2,0)=4.
        # All vertices have deg 2 and lie on the single triangle.
        # Vertex 0: s_0 = 1+4 = 5, triples = 5*1 = 5,
        #           triangle sum = w(0,1)+w(0,2) = 1+4 = 5 → 1.0.
        # Vertex 1: s_1 = 1+2 = 3, sum = 1+2 = 3 → 1.0.
        # Vertex 2: s_2 = 2+4 = 6, sum = 2+4 = 6 → 1.0.
        "origin": "constructed: weighted triangle hand-checked, all vertices = 1.0",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 4.0],
        "algo": "transitivity_barrat",
        "params": {},
        "expected": [1.0, 1.0, 1.0],
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

# ALGO-SP-001b: Dijkstra paths/parents — only `distances` is checked
# (parents/inbound_edges depend on tie-breaking).
DIJKSTRA_PATHS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_paths_c_triangle_with_shortcut",
        "origin": "constructed: triangle (1,4,2) with shortcut via vertex 1",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "dijkstra_paths",
        "params": {"source": 0},
        "expected": {"distances": [0.0, 1.0, 3.0]},
    },
]

# ALGO-SP-001b: single source-to-target convenience.
DIJKSTRA_PATH_TO_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_path_to_c_directed_path_with_long_shortcut",
        # Directed path 0->1->2->3 with unit weights, plus a long edge
        # 0->3 with weight 5. The Dijkstra path 0->1->2->3 wins.
        "origin": "constructed: directed P4 with heavier shortcut 0->3",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (0, 3)], directed=True
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "dijkstra_path_to",
        "params": {"source": 0, "target": 3},
        "expected": {"vertices": [0, 1, 2, 3], "edges": [0, 1, 2]},
    },
]

# ALGO-SP-001b: single-source distances with cutoff.
DIJKSTRA_CUTOFF_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_cutoff_c_path_5_cutoff_2_5",
        # Path 0-1-2-3-4 with unit weights and cutoff 2.5: vertices 0,1,2
        # reachable; 3 (dist 3) and 4 (dist 4) masked.
        "origin": "constructed: P5 unit weights with cutoff=2.5",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0, 1.0],
        "algo": "dijkstra_distances_cutoff",
        "params": {"source": 0, "cutoff": 2.5},
        "expected": [0.0, 1.0, 2.0, None, None],
    },
]

# ALGO-PR-020: is_dag (directed acyclic graph predicate). Source:
# properties/dag.c (lines 151-220). Kahn's topological peel.
IS_DAG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_dag_c_directed_chain_true",
        # 0 → 1 → 2 → 3: linear chain, no cycles.
        "origin": "constructed: directed P4 — DAG",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "algo": "is_dag",
        "params": {},
        "expected": True,
    },
    {
        "case": "is_dag_c_three_cycle_false",
        # 0 → 1 → 2 → 0: 3-cycle.
        "origin": "constructed: directed 3-cycle — not a DAG",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "is_dag",
        "params": {},
        "expected": False,
    },
]

# ALGO-CORE-001e: is_same_graph (structural equality). Source:
# graph/type_indexededgelist.c (lines 1947-2003). Compares two
# graphs as labelled vertex/edge sets — not isomorphism.
IS_SAME_GRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_same_graph_c_edge_order_differs_same",
        # Two graphs with the same vertex/edge sets in different
        # insertion orders are the same.
        "origin": "constructed: same edges, different order ⇒ same",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "is_same_graph",
        "params": {
            "other": {
                "n": 3,
                "edges": [[1, 2], [0, 1]],
                "directed": False,
            }
        },
        "expected": True,
    },
    {
        "case": "is_same_graph_c_directed_vs_undirected_not_same",
        "origin": "constructed: directed vs undirected, same edges ⇒ not same",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1)], directed=True
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

# ALGO-CC-032: Site percolation (vertex activation). Source:
# connectivity/percolation.c (lines 328-410). Each vertex activates
# in order; the connecting edges to already-activated neighbors
# percolate (self-loops count twice, parallels count separately).
SITE_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "site_perc_c_chain_natural_order",
        # Path 0-1-2-3, activate in id order.
        "origin": "constructed: P4 activated in id order",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "site_percolation",
        "params": {"vertex_order": [0, 1, 2, 3]},
        "expected": {
            "giant_size": [1, 2, 3, 4],
            "edge_count": [0, 1, 2, 3],
        },
    },
    {
        "case": "site_perc_c_triangle_jumps_at_third",
        # Triangle K_3, activate 0, 1, 2.
        "origin": "constructed: triangle — vertex 2 closes both extra edges",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "algo": "site_percolation",
        "params": {"vertex_order": [0, 1, 2]},
        "expected": {
            "giant_size": [1, 2, 3],
            "edge_count": [0, 1, 3],
        },
    },
]

# ALGO-CC-031: Bond percolation. Resolves the percolation sequence
# from edge ids into a Graph. Source: connectivity/percolation.c
# (lines 214-265). Wraps edgelist_percolation after edge lookup.
BOND_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bond_perc_c_natural_order_chain",
        # Path 0-1-2-3, edges added in id order — same curve as the
        # equivalent edgelist_percolation case.
        "origin": "constructed: P4 in natural id order",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "bond_percolation",
        "params": {"edge_order": [0, 1, 2]},
        "expected": {
            "giant_size": [2, 3, 4],
            "vertex_count": [2, 3, 4],
        },
    },
    {
        "case": "bond_perc_c_reordered_middle_first",
        # Same graph as bond_perc_c_natural_order_chain but edge ids
        # added in [1, 0, 2]: middle edge first, then left, then right.
        # The percolation curve depends on order.
        "origin": "constructed: P4 with middle edge added first",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "bond_percolation",
        "params": {"edge_order": [1, 0, 2]},
        "expected": {
            "giant_size": [2, 3, 4],
            "vertex_count": [2, 3, 4],
        },
    },
]

# ALGO-CC-030: Edge-list percolation. Reads the percolation sequence
# from the graph's edge list (insertion order). Source:
# connectivity/percolation.c (lines 105-180).
EDGELIST_PERCOLATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edgelist_perc_c_chain_growth",
        # 0-1, 1-2, 2-3, 3-4 → giant grows 2,3,4,5; vertices added at every step.
        "origin": "constructed: P5 chain — linear growth",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False
        ),
        "algo": "edgelist_percolation",
        "params": {},
        "expected": {
            "giant_size": [2, 3, 4, 5],
            "vertex_count": [2, 3, 4, 5],
        },
    },
    {
        "case": "edgelist_perc_c_two_components_then_join",
        "origin": "constructed: two pairs joined by a bridge edge — phase transition",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3), (1, 2)], directed=False
        ),
        "algo": "edgelist_percolation",
        "params": {},
        "expected": {
            "giant_size": [2, 2, 4],
            "vertex_count": [2, 4, 4],
        },
    },
]

# ALGO-SP-014: Single-source widest-paths SPT (widths + parents +
# inbound_edges). Source: widest_paths.c (lines 102-322).
# Fixtures encode source's width as null (Infinity convention).
WIDEST_PATHS_SPT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_paths_c_triangle_spt",
        # Triangle (1, 4, 2): widest 0→1 routes via 2 (bottleneck 2),
        # widest 0→2 direct (width 4).
        "origin": "constructed: triangle (1,4,2) — SPT with shortcut at 2",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "widest_paths",
        "params": {"source": 0},
        "expected": {
            "widths": [None, 2.0, 4.0],
            "parents": [None, 2, 0],
            "inbound_edges": [None, 2, 1],
        },
    },
    {
        "case": "widest_paths_c_unreachable_components",
        "origin": "constructed: two disjoint edges — half the vertices unreachable",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [3.0, 7.0],
        "algo": "widest_paths",
        "params": {"source": 0},
        "expected": {
            "widths": [None, 3.0, None, None],
            "parents": [None, 0, None, None],
            "inbound_edges": [None, 0, None, None],
        },
    },
]

# ALGO-SP-013: Multi-target widest paths (single source).
# Returns one Option<(vertices, edges)> per target, in the order
# the targets were given. Source: widest_paths.c (lines 102-322).
WIDEST_PATHS_TO_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_paths_to_c_triangle_two_targets",
        # Triangle (1, 4, 2): 0→1 goes via 2 (bottleneck 2), 0→2 direct (4).
        "origin": "constructed: triangle (1,4,2) — two targets, shortcut + direct",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "widest_paths_to",
        "params": {"from": 0, "targets": [1, 2]},
        "expected": [
            {"vertices": [0, 2, 1], "edges": [1, 2]},
            {"vertices": [0, 2], "edges": [1]},
        ],
    },
    {
        "case": "widest_paths_to_c_mixed_reachability",
        "origin": "constructed: two components; mid target unreachable",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 1.0],
        "algo": "widest_paths_to",
        "params": {"from": 0, "targets": [1, 2]},
        "expected": [{"vertices": [0, 1], "edges": [0]}, None],
    },
]

# ALGO-SP-012: Floyd-Warshall-based all-pairs widest widths matrix.
# Source: widest_paths.c (lines 451-555). All-pairs matrix; diagonal
# entries are +∞ by convention so we encode them as null in fixtures
# and the conformance runner skips diagonal.
WIDEST_PATH_WIDTHS_FW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_fw_c_triangle_all_pairs",
        # Same triangle (1, 4, 2): all-pairs widest widths must match
        # what the Dijkstra variant produces from every source.
        "origin": "constructed: triangle (1,4,2) — all-pairs widest matrix",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "widest_path_widths_floyd_warshall",
        "params": {},
        "expected": [
            [None, 2.0, 4.0],
            [2.0, None, 2.0],
            [4.0, 2.0, None],
        ],
    },
    {
        "case": "widest_fw_c_unreachable_components",
        # Two components: 0-1 with weight 5, 2-3 with weight 7.
        # Cross-component pairs are unreachable.
        "origin": "constructed: 4 vertices, 2 disjoint edges",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [5.0, 7.0],
        "algo": "widest_path_widths_floyd_warshall",
        "params": {},
        "expected": [
            [None, 5.0, None, None],
            [5.0, None, None, None],
            [None, None, None, 7.0],
            [None, None, 7.0, None],
        ],
    },
]

# ALGO-SP-011: Widest-path single source-to-target (returns the
# path itself, not just its width). Source: widest_paths.c (lines 102-322).
# Conformance compares only the bottleneck width along the returned
# path, not vertex identity (different tie-breaking is allowed).
WIDEST_PATH_GET_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_get_c_chain_unit_widths",
        # Path 0-1-2-3 unit widths; only one widest path exists.
        "origin": "constructed: P4 unit widths — unique widest path",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0],
        "algo": "widest_path",
        "params": {"from": 0, "to": 3},
        "expected": {"vertices": [0, 1, 2, 3], "edges": [0, 1, 2]},
    },
    {
        "case": "widest_get_c_unreachable_yields_null",
        "origin": "constructed: 4 vertices, 2 disjoint edges — no 0→2 path",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 1.0],
        "algo": "widest_path",
        "params": {"from": 0, "to": 2},
        "expected": None,
    },
]

# ALGO-SP-010: Widest-path widths (single source). Bottleneck on the
# best (max-min) path from source to each vertex. Source: widest_paths.c.
# Source's own width is convention-infinite; encoded as null in fixtures
# and skipped by the conformance runner.
WIDEST_PATH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "widest_c_triangle_direct_beats_chain",
        # Triangle (1, 4, 2). Source 0: widest 0→1 = min(4, 2) = 2
        # (via 0-2-1, not direct edge weight 1). Widest 0→2 = 4 (direct).
        "origin": "constructed: triangle (1,4,2) — widest paths via shortcut",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "widest_path_widths",
        "params": {"source": 0},
        "expected": [None, 2.0, 4.0],  # null at source position
    },
    {
        "case": "widest_c_chain_bottleneck_5_1_3",
        # Path 0-1-2-3 with weights (5, 1, 3). Bottleneck of best
        # path from 0: w[1]=5, w[2]=min(5,1)=1, w[3]=min(5,1,3)=1.
        "origin": "constructed: P4 weights (5,1,3) — bottleneck shrinks at edge 2",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "graph_weights": [5.0, 1.0, 3.0],
        "algo": "widest_path_widths",
        "params": {"source": 0},
        "expected": [None, 5.0, 1.0, 1.0],
    },
]

# ALGO-SP-003: Johnson all-pairs shortest distances. Used when many
# sources are needed AND graph has negative weights. Source:
# distances_johnson.c.
JOHNSON_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "johnson_c_directed_diamond_negative_edge",
        # Same diamond as the BF fixture: 0→1 (3), 0→2 (1), 1→3 (-2), 2→3 (4).
        # All-pairs matrix from upstream's Johnson.
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
        "case": "johnson_c_undirected_triangle_fast_path",
        # Non-negative weights ⇒ Johnson short-circuits to Dijkstra.
        "origin": "constructed: undirected triangle (1,4,2) — fast path",
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

# ALGO-SP-002: Bellman-Ford single-source distances. Handles negative
# weights that would break Dijkstra. Source: distances_bellman_ford.c.
BELLMAN_FORD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bellman_ford_c_directed_negative_edge",
        # Directed 0→1 (3), 0→2 (1), 1→3 (-2), 2→3 (4).
        # BF distances from 0: [0, 3, 1, 1] — the negative edge 1→3
        # makes 0→1→3 cheaper than 0→2→3.
        "origin": "constructed: directed K4-ish with one negative edge",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 3), (2, 3)], directed=True
        ),
        "graph_weights": [3.0, 1.0, -2.0, 4.0],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 3.0, 1.0, 1.0],
    },
    {
        "case": "bellman_ford_c_triangle_positive_matches_dijkstra",
        "origin": "constructed: triangle (1,4,2) — should agree with Dijkstra",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 4.0, 2.0],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 1.0, 3.0],
    },
    {
        "case": "bellman_ford_c_unreachable_component",
        "origin": "constructed: two disconnected components, source in first",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "graph_weights": [1.5, -0.5],
        "algo": "bellman_ford_distances",
        "params": {"source": 0},
        "expected": [0.0, 1.5, None, None],
    },
]

# ALGO-SP-001c: mode-aware distances variant. IN-mode reverses
# reachability on directed graphs.
DIJKSTRA_DIST_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_dist_mode_c_directed_path_in",
        # Directed path 0→1→2 with weights (1, 2). IN-mode from 2:
        # 2 reaches 1 (cost 2) and 0 (cost 3) by walking edges in reverse.
        "origin": "constructed: directed P3 (1,2), IN mode from sink",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "graph_weights": [1.0, 2.0],
        "algo": "dijkstra_distances_with_mode",
        "params": {"source": 2, "mode": "in"},
        "expected": [3.0, 2.0, 0.0],
    },
]

# ALGO-SP-001c: all-shortest-paths variant. The expected payload
# carries `distances` and `nrgeo` (path counts) — path enumeration
# itself is order-dependent and not checked.
DIJKSTRA_ASP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dijkstra_asp_c_diamond_two_geodesics",
        # Diamond 0-1-3 / 0-2-3, all weights 1: 2 distinct shortest
        # paths to vertex 3.
        "origin": "constructed: diamond unit weights, two geodesics to 3",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 3), (2, 3)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0, 1.0],
        "algo": "dijkstra_all_shortest_paths",
        "params": {"source": 0, "mode": "out"},
        "expected": {"distances": [0.0, 1.0, 1.0, 2.0], "nrgeo": [1, 1, 1, 2]},
    },
]

# ALGO-SP-005: A* shortest path. With null heuristic, A* ≡ Dijkstra
# single-source single-target. Conformance compares the full vertex+edge
# chain (matches the dijkstra_path_to convention).
ASTAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "a_star_path_c_directed_p4_with_long_shortcut",
        # Directed P4 0→1→2→3 with unit weights plus a long edge 0→3
        # weight 5: best path is the chain through every vertex.
        "origin": "constructed: directed P4 with heavier shortcut 0→3",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (0, 3)], directed=True
        ),
        "graph_weights": [1.0, 1.0, 1.0, 5.0],
        "algo": "a_star_path",
        "params": {"source": 0, "target": 3, "mode": "out"},
        "expected": {"vertices": [0, 1, 2, 3], "edges": [0, 1, 2]},
    },
]

# ALGO-SP-021..023 weighted: ecc / rad / diam Dijkstra-based.
ECC_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_weighted_c_path_3_weights_1_2_5",
        # Undirected path 0-1-2 weights (1, 2.5): ecc = [3.5, 2.5, 3.5].
        "origin": "constructed: P3 with weights (1, 2.5)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 2.5],
        "algo": "eccentricity_weighted_with_mode",
        "params": {"mode": "all"},
        "expected": [3.5, 2.5, 3.5],
    },
]

RAD_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_weighted_c_path_3_weights_1_2_5",
        # Same P3: radius = min ecc = 2.5.
        "origin": "constructed: P3 with weights (1, 2.5), radius = 2.5",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 2.5],
        "algo": "radius_weighted_with_mode",
        "params": {"mode": "all"},
        "expected": 2.5,
    },
]

DIAM_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_weighted_c_path_3_weights_1_2_5",
        # Same P3: diameter = max ecc = 3.5.
        "origin": "constructed: P3 with weights (1, 2.5), diameter = 3.5",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "graph_weights": [1.0, 2.5],
        "algo": "diameter_weighted_with_mode",
        "params": {"mode": "all"},
        "expected": 3.5,
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

# ALGO-PR-006d: Directed weighted assortativity. python-igraph 0.11
# has no Python-level weighted assortativity API, so non-unit-weight
# fixtures use hand-computed reference values from the upstream
# Pearson formula (Σ w·(s_out_u·s_in_v) etc.). Same convention as the
# undirected weighted PR-006b fixtures.
ASSORT_DIR_W_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_degree_directed_weighted_c_chain_with_branch_unit",
        # Same chain with branch (0→1, 1→2, 0→2) but with unit weights
        # — formula collapses to the unweighted directed PR-006c case
        # → r = -0.5. Cross-validates the directed-weighted impl.
        "origin": "constructed: 0→1, 1→2, 0→2 (chain with branch) unit weights",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (0, 2)],
            directed=True,
        ),
        "graph_weights": [1.0, 1.0, 1.0],
        "algo": "assortativity_degree_directed_weighted",
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

UNION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "union_c_directed_with_loop_disjoint_endpoints",
        # Mirrors references/igraph/tests/unit/igraph_union.c "BINARY VERSION":
        # left  = 0→1, 1→2, 2→2 (loop), 2→3
        # right = 0→1, 1→2, 2→2 (loop), 2→4
        # vcount = max(4, 5) = 5; per ordered pair max multiplicity → 5 edges.
        "origin": "references/igraph/tests/unit/igraph_union.c (BINARY VERSION)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 2), (2, 3)], directed=True
        ),
        "algo": "union",
        "params": {
            "right_graph": {
                "n": 5,
                "edges": [[0, 1], [1, 2], [2, 2], [2, 4]],
                "directed": True,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 5,
            "directed": True,
            "edges": [[0, 1], [1, 2], [2, 2], [2, 3], [2, 4]],
        },
    },
]

INTERSECTION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "intersection_c_directed_with_loop_disjoint_endpoints",
        # Mirrors the same `igraph_union.c` BINARY VERSION graphs but
        # asks for the intersection. Common ordered pairs (and their
        # min multiplicity): (0,1), (1,2), (2,2). Pairs unique to one
        # side (left's (2,3) and right's (2,4)) drop out. vcount =
        # max(4, 5) = 5 to match upstream's "common edges, larger vertex
        # set" contract.
        "origin": "constructed (mirrors igraph_union.c BINARY VERSION inputs)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 2), (2, 3)], directed=True
        ),
        "algo": "intersection",
        "params": {
            "right_graph": {
                "n": 5,
                "edges": [[0, 1], [1, 2], [2, 2], [2, 4]],
                "directed": True,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 5,
            "directed": True,
            "edges": [[0, 1], [1, 2], [2, 2]],
        },
    },
]

DIFFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "difference_c_directed_with_loop_disjoint_endpoints",
        # Same igraph_union.c BINARY VERSION inputs, queried for
        # difference (orig \ sub). Per directed pair:
        #   (0,1):1−1=0, (1,2):1−1=0, (2,2):1−1=0, (2,3):1−0=1
        #   (2,4) is not a key in orig, so it is ignored.
        # vcount = orig.vcount() = 4 (asymmetric — unlike union /
        # intersection which take max).
        "origin": "constructed (mirrors igraph_union.c BINARY VERSION inputs)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 2), (2, 3)], directed=True
        ),
        "algo": "difference",
        "params": {
            "right_graph": {
                "n": 5,
                "edges": [[0, 1], [1, 2], [2, 2], [2, 4]],
                "directed": True,
                "weights": None,
            }
        },
        "expected": {
            "vcount": 4,
            "directed": True,
            "edges": [[2, 3]],
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

ECC_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ecc_with_mode_c_directed_path4_in",
        # Directed path 0→1→2→3, queried under IGRAPH_IN. Reverse-BFS
        # from each vertex: from 0 reaches nothing (ecc=0); from 1
        # reaches 0 (ecc=1); from 2 reaches 1,0 (ecc=2); from 3 reaches
        # 2,1,0 (ecc=3). Expected: [0,1,2,3].
        "origin": "constructed: directed P4 — IN-mode reverses BFS direction",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "algo": "eccentricity_with_mode",
        "params": {"mode": "in"},
        "expected": [0, 1, 2, 3],
    },
]

RAD_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "radius_with_mode_c_directed_path4_in",
        # Same directed P4 under IN-mode. Min eccentricity over the
        # vector [0,1,2,3] is 0 (vertex 0 has no incoming edges).
        "origin": "constructed: directed P4 — IN-mode min ecc = 0 (source vertex)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "algo": "radius_with_mode",
        "params": {"mode": "in"},
        "expected": 0,
    },
]

DIAM_MODE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "diameter_with_mode_c_directed_path4_in",
        # Same directed P4 under IN-mode. Max ecc is 3 (vertex 3 reaches
        # all earlier vertices via reverse BFS).
        "origin": "constructed: directed P4 — IN-mode max ecc = 3 (longest reverse BFS)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=True
        ),
        "algo": "diameter_with_mode",
        "params": {"mode": "in"},
        "expected": 3,
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
    "is_dag": IS_DAG_MANIFEST,
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
    "reciprocity": RECIP_MANIFEST,
    "avg_nearest_neighbor_degree": KNN_MANIFEST,
    "avg_nearest_neighbor_degree_weighted": KNN_W_MANIFEST,
    "knnk": KNNK_MANIFEST,
    "knnk_weighted": KNNK_W_MANIFEST,
    "assortativity_degree": ASSORT_MANIFEST,
    "transitivity_barrat": TRANS_BARRAT_MANIFEST,
    "decompose": DECOMPOSE_MANIFEST,
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
