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

EIGEN_W_MANIFEST: List[Dict[str, Any]] = [
    {
        # Upstream golden test (igraph_eigenvector_centrality.c):
        # K_{1,4} star with unit weights — λ=2, vec=[1, 0.5, 0.5, 0.5, 0.5].
        "case": "eigenvector_w_c_star_unit",
        "origin": "igraph_eigenvector_centrality.c — weighted K_{1,4} with unit weights",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (0, 2), (0, 3), (0, 4)], directed=False
        ),
        "graph_weights": [1.0, 1.0, 1.0, 1.0],
        "algo": "eigenvector_centrality_weighted",
        "params": {},
        "expected": {
            "vector": [1.0, 0.5, 0.5, 0.5, 0.5],
            "eigenvalue": 2.0,
        },
    },
]

EIGEN_DIR_MANIFEST: List[Dict[str, Any]] = [
    {
        # Upstream golden test (igraph_eigenvector_centrality.c) —
        # directed 4-cycle + chord 1→3 with mode=OUT. Real-root
        # eigenvalue ≈ 1.220744, max-1 vec ≈ [0.819, 0.671, 0.550, 1.0].
        "case": "eigenvector_dir_c_cycle_chord_out",
        "origin": "igraph_eigenvector_centrality.c — directed 4-cycle+chord, mode=OUT",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (3, 0), (1, 3)],
            directed=True,
        ),
        "algo": "eigenvector_centrality_directed",
        "params": {"mode": "out"},
        # Reference values from python-igraph ARPACK on this exact graph.
        # Our shifted-power-iter agrees to ~1e-12 (within json_approx_eq
        # relative tolerance).
        "expected": {
            "vector": [
                0.8191725133961644,
                0.6710436067037893,
                0.5497004779019702,
                1.0,
            ],
            "eigenvalue": 1.2207440846057593,
        },
    },
]

HITS_MANIFEST: List[Dict[str, Any]] = [
    {
        # Mirrors the "Three vertices, no links" case from upstream
        # tests/unit/hub_and_authority.c: a directed graph with no
        # edges falls back to the all-ones convention.
        "case": "hits_c_directed_no_edges_ones",
        "origin": "igraph_hub_and_authority.c — 'Three vertices, no links' case",
        "graph_factory": lambda: ig.Graph(n=3, edges=[], directed=True),
        "algo": "hub_and_authority_scores",
        "params": {},
        "expected": {
            "hub": [1.0, 1.0, 1.0],
            "authority": [1.0, 1.0, 1.0],
            "eigenvalue": 0.0,
        },
    },
    {
        # Mirrors the "Two hubs and one authority" case from upstream
        # tests/unit/hub_and_authority.c (unweighted slice — ARPACK
        # weighted variant ships with PR-017b).
        "case": "hits_c_two_hubs_one_authority",
        "origin": "igraph_hub_and_authority.c — 'Two hubs and one authority' (unweighted)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 2), (1, 2)], directed=True
        ),
        "algo": "hub_and_authority_scores",
        "params": {},
        "expected": {
            "hub": [1.0, 1.0, 0.0],
            "authority": [0.0, 0.0, 1.0],
            "eigenvalue": 2.0,
        },
    },
]

HITS_W_MANIFEST: List[Dict[str, Any]] = [
    {
        # Same "Two hubs and one authority" topology as the unweighted
        # C fixture, but driven through the weighted code path with
        # unit weights — must produce the same result.
        "case": "hits_w_c_two_hubs_one_authority_unit",
        "origin": "igraph_hub_and_authority.c — 'Two hubs and one authority' weighted with unit weights",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 2), (1, 2)], directed=True
        ),
        "graph_weights": [1.0, 1.0],
        "algo": "hub_and_authority_scores_weighted",
        "params": {},
        "expected": {
            "hub": [1.0, 1.0, 0.0],
            "authority": [0.0, 0.0, 1.0],
            "eigenvalue": 2.0,
        },
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

# ALGO-PR-022: is_acyclic (predicate). Source: properties/trees.c
# (lines 753-762). Delegates to is_dag for directed; union-find
# over edges for undirected (cycle ⇔ second edge re-connects two
# already-connected vertices).
IS_ACYCLIC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_acyclic_c_undirected_tree_true",
        "origin": "constructed: undirected P4 — tree, acyclic",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "is_acyclic",
        "params": {},
        "expected": True,
    },
    {
        "case": "is_acyclic_c_directed_dag_true",
        "origin": "constructed: directed P3 — DAG, acyclic",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "algo": "is_acyclic",
        "params": {},
        "expected": True,
    },
]

# ALGO-PR-023: is_tree (predicate). Source: properties/trees.c
# (lines 251-392). Returns true iff `vcount-1` edges, all reachable
# from a chosen root via DFS in the requested orientation.
IS_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_tree_c_undirected_path_true",
        "origin": "constructed: undirected P4 — tree, root 0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "is_tree",
        "params": {"mode": "all"},
        "expected": True,
    },
    {
        "case": "is_tree_c_directed_out_arborescence_true",
        "origin": "constructed: 0→1, 0→2, 1→3 — out-tree rooted at 0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 3)], directed=True
        ),
        "algo": "is_tree",
        "params": {"mode": "out"},
        "expected": True,
    },
]

# ALGO-PR-024: is_forest (predicate + roots). Source: properties/
# trees.c (lines 520-725). Returns {is_forest, roots[]}; roots are
# the per-tree starting vertices (in-degree-0 for OUT, out-degree-0
# for IN, lowest-id-per-component for ALL/undirected).
IS_FOREST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_forest_c_undirected_two_components_true",
        "origin": "constructed: 0-1 ⊔ 2-3-4 — 2 disjoint trees",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (2, 3), (3, 4)], directed=False
        ),
        "algo": "is_forest",
        "params": {"mode": "all"},
        "expected": {"is_forest": True, "roots": [0, 2]},
    },
    {
        "case": "is_forest_c_directed_v_pattern_not_out_forest_false",
        "origin": "constructed: 0→2, 1→2 — vertex 2 has in-degree 2",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 2), (1, 2)], directed=True
        ),
        "algo": "is_forest",
        "params": {"mode": "out"},
        "expected": {"is_forest": False, "roots": []},
    },
]

# ALGO-PR-016: is_complete. Source: properties/complete.c (lines
# 43-155). Bool predicate: every distinct pair adjacent. Null and
# singleton are complete; directed graphs need both arcs per pair.
IS_COMPLETE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_complete_c_k4_undirected_true",
        "origin": "constructed: K_4 — every pair adjacent",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "algo": "is_complete",
        "params": {},
        "expected": True,
    },
    {
        "case": "is_complete_c_path_p4_undirected_false",
        "origin": "constructed: path 0-1-2-3 — endpoints not adjacent",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "is_complete",
        "params": {},
        "expected": False,
    },
]

# ALGO-PR-027: neighborhood_size. Source:
# tests/unit/igraph_neighborhood_size.c (.out file). Two fixtures
# from the upstream test driver: the all-mode order-1 case on the
# directed multigraph and the OUT-mode mindist-2 infinite-order case.
NEIGHBORHOOD_SIZE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "neighborhood_size_c_directed_loops_order_1_all",
        "origin": "tests/unit/igraph_neighborhood_size.c — directed n=6 multigraph, order=1 mode=ALL",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 1), (1, 3), (2, 0), (2, 3), (3, 4), (3, 4)],
            directed=True,
        ),
        "algo": "neighborhood_size",
        "params": {"order": 1, "mode": "all", "mindist": 0},
        "expected": [3, 3, 3, 4, 2, 1],
    },
    {
        "case": "neighborhood_size_c_directed_loops_infinite_mindist_2_out",
        "origin": "tests/unit/igraph_neighborhood_size.c — directed n=6, order=infinite mindist=2 mode=OUT",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 1), (1, 3), (2, 0), (2, 3), (3, 4), (3, 4)],
            directed=True,
        ),
        "algo": "neighborhood_size",
        "params": {"order": -1, "mode": "out", "mindist": 2},
        "expected": [2, 1, 2, 0, 0, 0],
    },
]

# ALGO-PR-027b: neighborhood (vertex lists). Source:
# tests/unit/igraph_neighborhood.c (.out file). Sorted per-vertex lists
# from the same directed n=6 multigraph used by neighborhood_size.
NEIGHBORHOOD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "neighborhood_c_directed_loops_order_1_all",
        "origin": "tests/unit/igraph_neighborhood.c — directed n=6 multigraph, order=1 mode=ALL",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 1), (1, 3), (2, 0), (2, 3), (3, 4), (3, 4)],
            directed=True,
        ),
        "algo": "neighborhood",
        "params": {"order": 1, "mode": "all", "mindist": 0},
        # Sorted lists; the C .out shows BFS-order ((0 1 2), (1 0 3), ...).
        "expected": [
            [0, 1, 2],
            [0, 1, 3],
            [0, 2, 3],
            [1, 2, 3, 4],
            [3, 4],
            [5],
        ],
    },
    {
        "case": "neighborhood_c_directed_loops_order_2_mindist_2_out",
        "origin": "tests/unit/igraph_neighborhood.c — directed n=6, order=2 mindist=2 mode=OUT",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 1), (1, 3), (2, 0), (2, 3), (3, 4), (3, 4)],
            directed=True,
        ),
        "algo": "neighborhood",
        "params": {"order": 2, "mode": "out", "mindist": 2},
        # C .out: 0:(3) 1:(4) 2:(1 4) 3:() 4:() 5:()
        "expected": [[3], [4], [1, 4], [], [], []],
    },
]

# ALGO-PR-021: topological_sorting. Source: properties/dag.c
# (lines 54-123). Kahn's peel, recording the popped order.
TOPOLOGICAL_SORTING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "topo_sort_c_linear_chain_out",
        # 0 → 1 → 2: unique OUT-mode order.
        "origin": "constructed: directed P3 — unique OUT topological order",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "algo": "topological_sorting",
        "params": {"mode": "out"},
        "expected": [0, 1, 2],
    },
    {
        "case": "topo_sort_c_linear_chain_in",
        # Same chain, IN mode reverses it.
        "origin": "constructed: directed P3 — IN mode reverses",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "algo": "topological_sorting",
        "params": {"mode": "in"},
        "expected": [2, 1, 0],
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

# ALGO-PR-028: convergence_degree. Source:
# references/igraph/tests/unit/igraph_convergence_degree.c.
# Per-edge value in [-1, 1] (directed) or [0, 1] (undirected)
# measuring whether shortest paths through the edge originate from
# a larger or smaller vertex set than they terminate in.
CONVERGENCE_DEGREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "convergence_degree_c_undirected_two_triangles",
        # Reproduces the first .out test case verbatim.
        "origin": (
            "references/igraph/tests/unit/igraph_convergence_degree.c "
            "test 1: undirected n=7, two triangles joined by a bridge"
        ),
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (0, 3), (1, 2), (1, 3),
                (2, 3), (3, 4), (4, 5), (4, 6), (5, 6),
            ],
            directed=False,
        ),
        "algo": "convergence_degree",
        "params": {},
        "expected": [
            0.0, 0.0, 0.6, 0.0, 0.6, 0.6,
            1.0 / 7.0, 2.0 / 3.0, 2.0 / 3.0, 0.0,
        ],
    },
    {
        "case": "convergence_degree_c_directed_star",
        # Reproduces the second .out test case verbatim. Directed
        # graph; expected ordering matches python-igraph's stored
        # edge ids (insertion order in the factory).
        "origin": (
            "references/igraph/tests/unit/igraph_convergence_degree.c "
            "test 2: directed n=6, four leaves into hub then hub→sink"
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

COUNT_LOOPS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_loops_c_three_self_loops_mixed",
        # Graph with three self-loops and one normal edge.
        # references/igraph/src/properties/loops.c:igraph_count_loops semantics:
        # count edges where IGRAPH_FROM == IGRAPH_TO. Each parallel self-loop
        # counts separately.
        "origin": "constructed: 4 vertices, edges (0,0)(1,1)(2,2)(0,3); count_loops=3",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 0), (1, 1), (2, 2), (0, 3)], directed=False
        ),
        "algo": "count_loops",
        "params": {},
        "expected": 3,
    },
    {
        "case": "count_loops_c_no_loops",
        # Plain undirected path → no self-loops.
        "origin": "constructed: undirected path 0-1-2-3; count_loops=0",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "count_loops",
        "params": {},
        "expected": 0,
    },
]

COUNT_MULTIPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_multiple_c_undirected_two_parallel",
        # Three undirected edges; first two share canonical (0,1).
        # references/igraph/src/properties/multiplicity.c:igraph_count_multiple
        # semantics with IGRAPH_LOOPS_ONCE / IGRAPH_MULTIPLE: each edge's
        # entry is the size of the equivalence class of its endpoint pair.
        "origin": "constructed: undirected (0,1)(0,1)(1,2); multiplicity (sorted) = [1,2,2]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (0, 1), (1, 2)], directed=False
        ),
        "algo": "count_multiple",
        "params": {},
        "expected": [1, 2, 2],
    },
    {
        "case": "count_multiple_c_directed_mutual_pair_distinct",
        # Directed (0,1) and (1,0) are distinct pairs → multiplicity 1 each.
        "origin": "constructed: directed (0,1)(1,0); multiplicity = [1,1]",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (1, 0)], directed=True
        ),
        "algo": "count_multiple",
        "params": {},
        "expected": [1, 1],
    },
]

COUNT_ADJACENT_TRIANGLES_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_adjacent_triangles_c_k4_each_three",
        # references/igraph/src/properties/triangles.c:igraph_count_adjacent_triangles
        # K4: 4 triangles total, every vertex sits in 3 of them.
        "origin": "constructed: K4 (4 vertices, 6 edges); per-vertex count = [3,3,3,3]",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "count_adjacent_triangles",
        "params": {},
        "expected": [3, 3, 3, 3],
    },
    {
        "case": "count_adjacent_triangles_c_diamond_k4_minus_edge",
        # K4 minus edge (0,3); triangles (0,1,2) and (1,2,3).
        "origin": "constructed: K4 minus edge (0,3); per-vertex count = [1,2,2,1]",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "algo": "count_adjacent_triangles",
        "params": {},
        "expected": [1, 2, 2, 1],
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

# Louvain (ALGO-CO-002). Upstream reference C entry point:
# references/igraph/src/community/louvain.c
# `igraph_community_multilevel`. Louvain's exact partition depends on
# shuffle order, so we assert on the achievable modularity range and on
# the community count window, not on exact membership.
LOUVAIN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "louvain_c_karate_zachary",
        # Zachary's karate club: 34v 78e. The Louvain reference
        # implementation lands on Q ≈ 0.39..0.42 across shuffle orders;
        # the partition typically has 4 communities. Mirrors the C-side
        # use of Famous("Zachary") in references/igraph/examples/.
        "origin": "Famous('Zachary'); Louvain Q ≈ 0.39..0.42, k ≈ 4 "
        "(python-igraph community_multilevel cross-checked)",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "louvain",
        "params": {"resolution": 1.0},
        "expected": {
            "modularity_min": 0.38,
            "modularity_max": 0.43,
            "k_min": 2,
            "k_max": 6,
        },
    },
    {
        "case": "louvain_c_two_k4_bridge",
        # Two K4s joined by a single bridge edge — Louvain MUST split
        # the two cliques (the bridge contributes a single edge while
        # each K4 contributes 6). Reference Q = 0.4231 exactly.
        "origin": "constructed: two K4 + bridge (3,4); Louvain k=2, "
        "Q ≈ 0.4231",
        "graph_factory": lambda: ig.Graph(
            n=8,
            edges=[
                (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
                (4, 5), (4, 6), (4, 7), (5, 6), (5, 7), (6, 7),
                (3, 4),
            ],
            directed=False,
        ),
        "algo": "louvain",
        "params": {"resolution": 1.0},
        "expected": {
            "modularity_min": 0.40,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 3,
        },
    },
]

# Leiden (ALGO-CO-003). Upstream reference C entry point:
# references/igraph/src/community/leiden.c
# `igraph_community_leiden` / `igraph_community_leiden_simple`. Leiden
# is non-deterministic across implementations (different shuffle / RNG /
# tie-breaking strategies), so we assert on a Q-range and a k-window
# rather than an exact membership vector — same pattern as Louvain.
LEIDEN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "leiden_c_karate_zachary",
        "origin": "Famous('Zachary'); Leiden Modularity Q ≈ 0.39..0.45, "
        "k ≈ 4 (python-igraph community_leiden cross-checked, "
        "objective=\"modularity\")",
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
        "case": "leiden_c_two_k4_bridge",
        # Two K4s joined by a single bridge edge — Leiden must split
        # them with Q in the same range as Louvain (≈ 0.4231).
        "origin": "constructed: two K4 + bridge (3,4); Leiden k=2, "
        "Q ≈ 0.4231",
        "graph_factory": lambda: ig.Graph(
            n=8,
            edges=[
                (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
                (4, 5), (4, 6), (4, 7), (5, 6), (5, 7), (6, 7),
                (3, 4),
            ],
            directed=False,
        ),
        "algo": "leiden",
        "params": {"objective": "modularity", "resolution": 1.0},
        "expected": {
            "modularity_min": 0.40,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 3,
        },
    },
]

WALKTRAP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "walktrap_c_karate",
        # references/igraph/tests/unit/community_walktrap.c is the closest
        # upstream C test. Walktrap on Famous("Zachary") with steps=4
        # cuts at Q ≈ 0.35..0.42 with k ∈ [4, 6] (varies a tick with
        # tie-break across ports). Envelope kept wide.
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
        "case": "walktrap_c_two_k4_bridge",
        # Two K4s joined by a single bridge edge: Walktrap recovers the
        # split cleanly at k = 2 with Q ≈ 0.42.
        "origin": "constructed: two K4 + bridge (3,4); community_walktrap "
        "steps=4; k=2, Q ≈ 0.36..0.45",
        "graph_factory": lambda: ig.Graph(
            n=8,
            edges=[
                (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
                (4, 5), (4, 6), (4, 7), (5, 6), (5, 7), (6, 7),
                (3, 4),
            ],
            directed=False,
        ),
        "algo": "walktrap",
        "params": {"steps": 4},
        "expected": {
            "modularity_min": 0.35,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 2,
        },
    },
    {
        "case": "walktrap_c_ring6_weighted",
        # community_walktrap.out "Small weighted graph" case.
        # 6-ring with weights [1.0, 0.5, 0.25, 0.75, 1.25, 1.5];
        # steps=4 yields Q = 0.146259 at the best cut with k = 3.
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
        "case": "fastgreedy_c_karate",
        # references/igraph/tests/unit/igraph_community_fastgreedy.c
        # reports Q = 0.380671 with 3 communities on Famous("Zachary").
        # Tolerance kept wide to absorb tie-break differences across ports.
        "origin": "Famous('Zachary'); community_fastgreedy; "
        "C unit test Q = 0.380671, k = 3",
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
        "case": "fastgreedy_c_two_k5_bridge",
        # references/igraph/tests/unit/igraph_community_fastgreedy.c
        # reports Q = 0.452381 with k = 2 on two K5 + bridge (0,5).
        "origin": "two K5 + bridge (0,5); community_fastgreedy "
        "C unit test Q = 0.452381, k = 2",
        "graph_factory": lambda: ig.Graph(
            n=10,
            edges=[
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4),
                (2, 3), (2, 4), (3, 4),
                (5, 6), (5, 7), (5, 8), (5, 9),
                (6, 7), (6, 8), (6, 9),
                (7, 8), (7, 9), (8, 9),
                (0, 5),
            ],
            directed=False,
        ),
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
        "case": "eb_community_c_karate",
        # Famous("Zachary") karate club. Girvan-Newman edge-betweenness
        # routinely lands a partition with Q ≈ 0.40 ± 0.05 and k in [2, 5].
        # See Girvan & Newman PNAS 2002 Table I.
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
        "case": "eb_community_c_two_k4_bridge",
        # Two K4s joined by a single bridge edge. The bridge has the
        # largest betweenness and is removed first, cleanly splitting
        # into 2 clusters; Q ≈ 0.42.
        "origin": "constructed: two K4 + bridge (3,4); EB community "
        "k=2; Q ≈ 0.36..0.45",
        "graph_factory": lambda: ig.Graph(
            n=8,
            edges=[
                (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
                (4, 5), (4, 6), (4, 7), (5, 6), (5, 7), (6, 7),
                (3, 4),
            ],
            directed=False,
        ),
        "algo": "edge_betweenness_community",
        "params": {},
        "expected": {
            "modularity_min": 0.35,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 2,
        },
    },
    {
        "case": "eb_community_c_directed_path_6",
        # Directed 6-path (CO-006c): edge (2,3) is the unique max-
        # betweenness edge ⇒ first removal ⇒ {0,1,2}|{3,4,5}.
        # Directed-modularity envelope hand-checked at 8/25 = 0.32.
        "origin": "constructed: directed 6-path; EB community on directed graph; "
        "k=2; directed Q = 8/25 ≈ 0.32",
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

# Weighted edge-betweenness community (ALGO-CO-006b). Upstream reference
# C entry point: references/igraph/src/community/edge_betweenness.c
# `igraph_community_edge_betweenness(..., weights=&w, ...)`. The C test
# `igraph_community_edge_betweenness.c` exercises the unweighted path;
# the weighted path is covered by the python-igraph oracle (same
# algorithm with weights=...). Unit weights ≡ unweighted result.
EB_COMMUNITY_WEIGHTED_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "eb_community_weighted_c_karate_unit",
        # Famous("Zachary") with all unit weights — must reproduce the
        # unweighted Girvan-Newman dendrogram exactly. Q/k envelope is
        # identical to the unweighted slice.
        "origin": "Famous('Zachary'); weighted EB community, unit weights; "
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
        "case": "eb_community_weighted_c_two_k4_cheap_bridge",
        # Two K4s joined by a single bridge with weight 0.1; intra-clique
        # edges have weight 1.0. The cheap bridge sits on every
        # cross-component shortest path → first removed; best-Q under
        # weighted modularity keeps each K4 intact. Q range widens
        # slightly versus the unit case because the heavy intra-cluster
        # edges dominate m = Σ w_e.
        "origin": "two K4 + cheap-bridge (3,4) w=0.1; weighted EB community "
        "k=2; Q ∈ [0.30, 0.50]",
        "graph_factory": lambda: ig.Graph(
            n=8,
            edges=[
                (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
                (4, 5), (4, 6), (4, 7), (5, 6), (5, 7), (6, 7),
                (3, 4),
            ],
            directed=False,
        ),
        "graph_weights": [1.0] * 12 + [0.1],
        "algo": "edge_betweenness_community_weighted",
        "params": {},
        "expected": {
            "modularity_min": 0.30,
            "modularity_max": 0.50,
            "k_min": 2,
            "k_max": 3,
        },
    },
    {
        "case": "eb_community_weighted_c_directed_path_6_unit",
        # Directed 6-path with unit weights (CO-006c): identical
        # dendrogram to the unweighted directed slice.
        "origin": "constructed: directed 6-path; weighted EB community unit weights; "
        "k=2; directed-weighted Q = 8/25 ≈ 0.32",
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
        "case": "fluid_c_karate_k2",
        # Famous("Zachary") karate club with k=2. The natural split of
        # the karate club is along the instructor/officer cleavage; Q of
        # the Fluid partition lands around 0.36..0.40.
        "origin": "Famous('Zachary'); fluid_communities k=2; Q ∈ [0.20, 0.42]",
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
        "case": "fluid_c_two_k4_bridge_k2",
        # Two K4s joined by a single bridge edge. Fluid with k=2 cleanly
        # cuts the bridge; Q ≈ 0.42.
        "origin": "constructed: two K4 + bridge (3,4); fluid k=2; Q ≈ 0.36..0.45",
        "graph_factory": lambda: ig.Graph(
            n=8,
            edges=[
                (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
                (4, 5), (4, 6), (4, 7), (5, 6), (5, 7), (6, 7),
                (3, 4),
            ],
            directed=False,
        ),
        "algo": "fluid_communities",
        "params": {"k": 2},
        "expected": {
            "modularity_min": 0.35,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 2,
        },
    },
]

LPA_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lpa_c_karate_zachary",
        # Famous("Zachary") karate club. LPA is stochastic; modularity
        # of the partition typically lands in [0.30, 0.42] and k in
        # [2, 8] across the three variants.
        "origin": "Famous('Zachary'); label_propagation Q ∈ [0.20, 0.42], "
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
        "case": "lpa_c_two_k4_bridge",
        # Two K4s joined by a single bridge edge. The dominant-label
        # rule yields k = 2 almost surely; Q ≈ 0.42 by ground truth.
        "origin": "constructed: two K4 + bridge (3,4); LPA k=2, "
        "Q ≈ 0.36..0.45",
        "graph_factory": lambda: ig.Graph(
            n=8,
            edges=[
                (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
                (4, 5), (4, 6), (4, 7), (5, 6), (5, 7), (6, 7),
                (3, 4),
            ],
            directed=False,
        ),
        "algo": "label_propagation",
        "params": {},
        "expected": {
            "modularity_min": 0.35,
            "modularity_max": 0.45,
            "k_min": 2,
            "k_max": 3,
        },
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

# VF2 automorphism counts (self-comparison). Mirrors the upstream VF2 unit
# test references/igraph/tests/unit/igraph_isomorphic_vf2.c, which checks
# igraph_count_isomorphisms_vf2(ring, ring, ...) — the undirected ring(n)
# has 2n automorphisms (n rotations x 2 reflections) and a directed ring
# has n (rotations only). Graphs must be simple and loopless (VF2 rejects
# self-loops).
VF2_COUNT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_isomorphisms_vf2_c_ring6_undirected",
        "origin": "igraph_isomorphic_vf2.c: count_isomorphisms_vf2(ring, ring); undirected ring(n) has 2n automorphisms, n=6 -> 12",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)],
            directed=False,
        ),
        "algo": "count_isomorphisms_vf2",
        "params": {},
        "expected": 12,
    },
    {
        "case": "count_isomorphisms_vf2_c_ring4_directed",
        "origin": "igraph_isomorphic_vf2.c: directed ring has only rotations; directed ring(4) -> 4",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=True
        ),
        "algo": "count_isomorphisms_vf2",
        "params": {},
        "expected": 4,
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

GLOBAL_EFFICIENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "global_efficiency_c_k4",
        # references/igraph/src/paths/shortest_paths.c:igraph_global_efficiency
        # K4: every ordered pair at distance 1 → mean 1/d = 1.
        "origin": "constructed: K4; global_efficiency=1.0",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "global_efficiency",
        "params": {},
        "expected": 1.0,
    },
    {
        "case": "global_efficiency_c_path3",
        # 0-1-2: 6 ordered pairs. d=1 ×4, d=2 ×2 → sum = 4 + 1 = 5; /6.
        "origin": "constructed: undirected path 0-1-2; global_efficiency=5/6",
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
        "case": "local_efficiency_c_k4",
        # references/igraph/src/paths/shortest_paths.c:igraph_local_efficiency
        # K4: each vertex's neighbour set is K3, distances all 1 in
        # G\{v} → local efficiency = 1.0 at every vertex.
        "origin": "constructed: K4; per-vertex local_efficiency=[1,1,1,1]",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "local_efficiency",
        "params": {},
        "expected": [1.0, 1.0, 1.0, 1.0],
    },
    {
        "case": "local_efficiency_c_path3",
        # Path 0-1-2: vertex 1 has neighbours {0,2} disconnected in G\{1}
        # → 0; vertices 0, 2 have one neighbour each → 0.
        "origin": "constructed: path 0-1-2; per-vertex local_efficiency=[0,0,0]",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "local_efficiency",
        "params": {},
        "expected": [0.0, 0.0, 0.0],
    },
]

AVERAGE_LOCAL_EFFICIENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "average_local_efficiency_c_k4",
        # references/igraph/src/paths/shortest_paths.c:igraph_average_local_efficiency
        # K4: all per-vertex 1.0 → mean 1.0.
        "origin": "constructed: K4; average_local_efficiency=1.0",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "average_local_efficiency",
        "params": {},
        "expected": 1.0,
    },
    {
        "case": "average_local_efficiency_c_diamond",
        # Diamond 0-1, 0-2, 0-3, 1-2, 2-3: per-vertex local efficiency is
        # [5/6, 1, 5/6, 1] → mean = 11/12.
        "origin": "constructed: diamond; average_local_efficiency=11/12",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (0, 3), (1, 2), (2, 3)], directed=False
        ),
        "algo": "average_local_efficiency",
        "params": {},
        "expected": 11.0 / 12.0,
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

COMMUNITY_TO_MEMBERSHIP_MANIFEST: List[Dict[str, Any]] = [
    # `igraph_community_to_membership` in
    # references/igraph/src/community/community_misc.c. The C reference
    # has no dedicated unit test (the helper is exercised through
    # walktrap / fast_greedy_modularity / edge_betweenness_community
    # tests). The fixtures below mirror its documented contract on
    # small hand-constructed dendrograms: walk merges top-down,
    # assigning supercluster ids; densify with untouched leaves as
    # singletons. Expected membership/csize computed via python-igraph
    # `VertexDendrogram.as_clustering(n)` (semantically identical) and
    # then partition-equivalence-compared against the Rust impl
    # (cluster labels may differ).
    {
        "case": "community_to_membership_c_balanced_4_cut2",
        "origin": "C reference community_misc.c igraph_community_to_membership: "
        "4 leaves, balanced merges [[0,1],[2,3],[4,5]], steps=2 -> 2 clusters",
        "nodes": 4,
        "merges": [[0, 1], [2, 3], [4, 5]],
        "steps": 2,
        "expected": {"membership": [0, 0, 1, 1], "csize": [2, 2]},
    },
    {
        "case": "community_to_membership_c_chain_full_collapse",
        "origin": "C reference community_misc.c igraph_community_to_membership: "
        "4 leaves, chain merges [[0,1],[4,2],[5,3]], steps=3 -> 1 cluster of 4",
        "nodes": 4,
        "merges": [[0, 1], [4, 2], [5, 3]],
        "steps": 3,
        "expected": {"membership": [0, 0, 0, 0], "csize": [4]},
    },
]

COMPARE_COMMUNITIES_MANIFEST: List[Dict[str, Any]] = [
    # `igraph_compare_communities` in
    # references/igraph/src/community/community_misc.c. The C
    # reference ships a `community_comparison.out` printer but its
    # numeric outputs come from running the same algorithm under the
    # listed method. The fixtures below cover the closed-form values
    # for identical / partially overlapping / fully-disagreeing
    # partitions, each method gets at least one fixture.
    {
        "case": "compare_communities_c_identical_nmi_1",
        "origin": "C reference community_misc.c igraph_compare_communities: "
        "identical partitions on n=6 — NMI=1, VI=0, SJ=0, Rand=1, AR=1.",
        "comm1": [0, 0, 1, 1, 2, 2],
        "comm2": [7, 7, 3, 3, 9, 9],
        "method": "normalized_mutual_information",
        "expected": {"value": 1.0},
    },
    {
        "case": "compare_communities_c_full_disagreement_2x2",
        "origin": "C reference community_misc.c igraph_compare_communities: "
        "full-disagreement 2x2 confusion (n=4) — Rand index = 1/3.",
        "comm1": [0, 0, 1, 1],
        "comm2": [0, 1, 0, 1],
        "method": "rand",
        "expected": {"value": 1.0 / 3.0},
    },
]

SPLIT_JOIN_DISTANCE_MANIFEST: List[Dict[str, Any]] = [
    # `igraph_split_join_distance` in
    # references/igraph/src/community/community_misc.c. The C reference
    # returns the asymmetric (distance12, distance21) pair; the
    # symmetric scalar reported by `igraph_compare_communities` with
    # IGRAPH_COMMCMP_SPLIT_JOIN is the sum of the two components.
    {
        "case": "split_join_distance_c_subpartition_asymmetric",
        "origin": "C reference community_misc.c igraph_split_join_distance: "
        "comm1={{0,1},{2},{3}} is a sub-partition of comm2={{0,1,2},{3}} ⇒ d12=0, d21=1.",
        "comm1": [0, 0, 1, 2],
        "comm2": [0, 0, 0, 1],
        "expected": {"d12": 0, "d21": 1},
    },
    {
        "case": "split_join_distance_c_full_disagreement_2x2",
        "origin": "C reference community_misc.c igraph_split_join_distance: "
        "full-disagreement 2x2 confusion (n=4) — d12=d21=2, total=4 matches CM-015's SplitJoin.",
        "comm1": [0, 0, 1, 1],
        "comm2": [0, 1, 0, 1],
        "expected": {"d12": 2, "d21": 2},
    },
]

VORONOI_MANIFEST: List[Dict[str, Any]] = [
    # `igraph_voronoi` reference test at
    # references/igraph/tests/unit/igraph_voronoi.c. The .out file ships
    # canonical (deterministic) outputs for FIRST and LAST tiebreakers
    # on the disconnected-directed-multigraph and the unweighted karate
    # club. We do NOT extract the RANDOM tiebreaker because the C
    # default RNG (Mersenne Twister, seeded 42) and our SplitMix64 do
    # not produce identical tie selections.
    {
        "case": "voronoi_c_disconnected_directed_multigraph_first",
        "origin": "C reference paths/voronoi.c igraph_voronoi: "
        "disconnected directed multigraph, generators=[0,1], mode=OUT, FIRST tiebreaker",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[(0, 2), (1, 2), (2, 3), (3, 4), (5, 4), (6, 4), (2, 3), (1, 1)],
            directed=True,
        ),
        "algo": "voronoi",
        "params": {
            "generators": [0, 1],
            "mode": "out",
            "tiebreaker": "first",
        },
        "expected": {
            "membership": [0, 1, 0, 0, 0, None, None],
            "distances": [0.0, 0.0, 1.0, 2.0, 3.0, None, None],
        },
    },
    {
        "case": "voronoi_c_disconnected_directed_multigraph_last",
        "origin": "C reference paths/voronoi.c igraph_voronoi: "
        "disconnected directed multigraph, generators=[0,1], mode=OUT, LAST tiebreaker",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[(0, 2), (1, 2), (2, 3), (3, 4), (5, 4), (6, 4), (2, 3), (1, 1)],
            directed=True,
        ),
        "algo": "voronoi",
        "params": {
            "generators": [0, 1],
            "mode": "out",
            "tiebreaker": "last",
        },
        "expected": {
            "membership": [0, 1, 1, 1, 1, None, None],
            "distances": [0.0, 0.0, 1.0, 2.0, 3.0, None, None],
        },
    },
    {
        "case": "voronoi_c_karate_unweighted_first",
        "origin": "C reference paths/voronoi.c igraph_voronoi: "
        "Zachary karate club, generators=[0,32,24], mode=ALL, FIRST tiebreaker (unweighted)",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "voronoi",
        "params": {
            "generators": [0, 32, 24],
            "mode": "all",
            "tiebreaker": "first",
        },
        "expected": {
            "membership": [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0,
                0, 1, 0, 1, 0, 1, 1, 2, 2, 1, 2, 0, 1, 1, 0, 1, 1,
            ],
            "distances": [
                0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0,
                1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                1.0, 1.0, 0.0, 1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 0.0, 1.0,
            ],
        },
    },
    {
        "case": "voronoi_c_karate_unweighted_last",
        "origin": "C reference paths/voronoi.c igraph_voronoi: "
        "Zachary karate club, generators=[0,32,24], mode=ALL, LAST tiebreaker (unweighted)",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "voronoi",
        "params": {
            "generators": [0, 32, 24],
            "mode": "all",
            "tiebreaker": "last",
        },
        "expected": {
            "membership": [
                0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0,
                0, 1, 0, 1, 0, 1, 1, 2, 2, 1, 2, 2, 1, 1, 2, 1, 1,
            ],
            "distances": [
                0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0,
                1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                1.0, 1.0, 0.0, 1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 0.0, 1.0,
            ],
        },
    },
]

ECC_PR031_MANIFEST: List[Dict[str, Any]] = [
    # `igraph_ecc` reference test at
    # references/igraph/tests/unit/igraph_ecc.c. Expected values are
    # transcribed verbatim from `igraph_ecc.out` and re-cross-checked
    # by hand. NaN entries are encoded as JSON `null` (the conformance
    # runner converts NaN ↔ null both ways).
    {
        "case": "ecc_c_k5_k3_normalized",
        "origin": "references/igraph/tests/unit/igraph_ecc.out line 33: "
        "K_5, k=3, offset=false, normalize=true → every edge yields 1.0",
        "graph_factory": lambda: ig.Graph.Full(n=5, directed=False, loops=False),
        "algo": "ecc",
        "params": {"k": 3, "offset": False, "normalize": True},
        "expected": [1.0] * 10,
    },
    {
        "case": "ecc_c_k5_k4_normalized",
        "origin": "references/igraph/tests/unit/igraph_ecc.out line 38: "
        "K_5, k=4, offset=false, normalize=true → every edge yields 2/3",
        "graph_factory": lambda: ig.Graph.Full(n=5, directed=False, loops=False),
        "algo": "ecc",
        "params": {"k": 4, "offset": False, "normalize": True},
        "expected": [2.0 / 3.0] * 10,
    },
    {
        "case": "ecc_c_k5_with_loops_k3_normalized",
        "origin": "references/igraph/tests/unit/igraph_ecc.out line 43: "
        "K_5 with self-loops, k=3, offset=false, normalize=true",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[
                (0, 0), (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 1), (1, 2), (1, 3), (1, 4),
                (2, 2), (2, 3), (2, 4),
                (3, 3), (3, 4),
                (4, 4),
            ],
            directed=False,
        ),
        "algo": "ecc",
        "params": {"k": 3, "offset": False, "normalize": True},
        "expected": [
            None, 0.6, 0.6, 0.6, 0.6, None, 0.6, 0.6, 0.6, None,
            0.6, 0.6, None, 0.6, None,
        ],
    },
    {
        "case": "ecc_c_k5_with_loops_k4_normalized",
        "origin": "references/igraph/tests/unit/igraph_ecc.out line 48: "
        "K_5 with self-loops, k=4, offset=false, normalize=true",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[
                (0, 0), (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 1), (1, 2), (1, 3), (1, 4),
                (2, 2), (2, 3), (2, 4),
                (3, 3), (3, 4),
                (4, 4),
            ],
            directed=False,
        ),
        "algo": "ecc",
        "params": {"k": 4, "offset": False, "normalize": True},
        "expected": [
            None, 0.24, 0.24, 0.24, 0.24, None, 0.24, 0.24, 0.24, None,
            0.24, 0.24, None, 0.24, None,
        ],
    },
    {
        "case": "ecc_c_multigraph_k3_normalized",
        "origin": "references/igraph/tests/unit/igraph_ecc.out line 53: "
        "multigraph with loops + parallel edges, k=3, normalize=true",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1), (1, 2), (2, 0), (0, 1), (1, 3),
                (3, 4), (4, 0), (0, 5), (5, 5), (5, 5), (1, 4),
            ],
            directed=False,
        ),
        "algo": "ecc",
        "params": {"k": 3, "offset": False, "normalize": True},
        "expected": [0.5, 1.0, 1.0, 0.5, 1.0, 1.0, 0.5, 0.0, None, None, 1.0],
    },
]


# `igraph_rich_club_sequence` reference test at
# references/igraph/tests/unit/rich_club.c. Expected values are
# transcribed from `rich_club.out` and re-cross-checked by hand from
# exact rationals; NaN entries (trailing single-vertex / empty subgraph
# under `loops=false`) are encoded as JSON `null` (the runner converts
# NaN ↔ null both ways, mirroring the `ecc` convention above).
RICH_CLUB_MANIFEST: List[Dict[str, Any]] = [
    {
        # Test 3a — undirected, no self-loops, in-order vertex removal.
        # 7 vertices, 8 edges. Output denominators are
        # k*(k-1)/2 for k = 7,6,5,4,3,2,1.
        "case": "rich_club_c_undirected_no_loop_inorder",
        "origin": "rich_club.out Test 3a — undirected no-loop, vertex_order=[0..6]",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[(0, 3), (1, 3), (2, 3), (4, 3), (5, 3), (5, 6), (1, 2), (2, 5)],
            directed=False,
        ),
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [0, 1, 2, 3, 4, 5, 6],
            "normalized": True,
            "loops": False,
            "directed": False,
        },
        "expected": [8 / 21, 7 / 15, 5 / 10, 3 / 6, 1 / 3, 1.0, None],
    },
    {
        # Test 6a — directed, with one self-loop (4,4), in-order
        # vertex removal. 7 vertices, 9 edges. Denominator is n^2
        # (directed + loops), so the trailing single-vertex subgraph
        # yields 0 (no remaining edges), not NaN.
        "case": "rich_club_c_directed_loop_inorder",
        "origin": "rich_club.out Test 6a — directed loop, vertex_order=[0..6]",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 2), (1, 2), (2, 3), (1, 3), (3, 5),
                (3, 4), (5, 6), (6, 5), (4, 4),
            ],
            directed=True,
        ),
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [0, 1, 2, 3, 4, 5, 6],
            "normalized": True,
            "loops": True,
            "directed": True,
        },
        "expected": [9 / 49, 8 / 36, 6 / 25, 5 / 16, 3 / 9, 2 / 4, 0.0],
    },
    {
        # Test 7a — same graph as Test 3a but with all edge weights = 2.
        # Each rich-club coefficient is exactly double of Test 3a, with
        # the trailing NaN preserved (denominator 0 from k*(k-1)/2 at k=1).
        "case": "rich_club_c_weighted_double",
        "origin": "rich_club.out Test 7a — weighted (all weights = 2), vertex_order=[0..6]",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[(0, 3), (1, 3), (2, 3), (4, 3), (5, 3), (5, 6), (1, 2), (2, 5)],
            directed=False,
        ),
        "graph_weights": [2.0] * 8,
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [0, 1, 2, 3, 4, 5, 6],
            "normalized": True,
            "loops": False,
            "directed": False,
        },
        "expected": [16 / 21, 14 / 15, 10 / 10, 6 / 6, 2 / 3, 2.0, None],
    },
]


COMMUNITY_VORONOI_MANIFEST: List[Dict[str, Any]] = [
    # `igraph_community_voronoi` reference test at
    # references/igraph/tests/unit/igraph_community_voronoi.c. Expected
    # values are transcribed verbatim from
    # `igraph_community_voronoi.out`. The runner only asserts on
    # `generators` and the number of distinct community ids — the raw
    # membership labels depend on the RANDOM-tiebreaker outcome inside
    # `igraph_voronoi`, which differs between Mersenne Twister (C) and
    # our SplitMix64. Generator ordering is deterministic (driven by
    # local relative density), so it survives RNG changes.
    {
        "case": "community_voronoi_c_null",
        "origin": "references/igraph/tests/unit/igraph_community_voronoi.out: "
        "null graph (n=0) — empty membership + generators",
        "graph_factory": lambda: ig.Graph(n=0, edges=[], directed=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": -1.0},
        "expected": {"generators": [], "community_count": 0},
    },
    {
        "case": "community_voronoi_c_singleton",
        "origin": "references/igraph/tests/unit/igraph_community_voronoi.out: "
        "singleton (n=1) — single self-generator, single community",
        "graph_factory": lambda: ig.Graph(n=1, edges=[], directed=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": -1.0},
        "expected": {"generators": [0], "community_count": 1},
    },
    {
        "case": "community_voronoi_c_two_isolated_nodes",
        "origin": "references/igraph/tests/unit/igraph_community_voronoi.out: "
        "two isolated vertices — each its own generator + community",
        "graph_factory": lambda: ig.Graph(n=2, edges=[], directed=False),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": -1.0},
        "expected": {"generators": [0, 1], "community_count": 2},
    },
    {
        "case": "community_voronoi_c_zachary_auto_r",
        "origin": "references/igraph/tests/unit/igraph_community_voronoi.out: "
        "Zachary karate club, mode=ALL, r=-1 (auto-r) — generators = (33, 0, 24)",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "algo": "community_voronoi",
        "params": {"mode": "all", "r": -1.0},
        "expected": {"generators": [33, 0, 24], "community_count": 3},
    },
]


REINDEX_MEMBERSHIP_MANIFEST: List[Dict[str, Any]] = [
    # `igraph_reindex_membership` /
    # `igraph_i_reindex_membership_large` in
    # references/igraph/src/community/community_misc.c. C has no
    # dedicated unit test for this helper either; it ships as
    # supporting infrastructure for the leading-eigenvector and other
    # community algos that need contiguous 0..k-1 labels. The
    # fixtures below mirror the documented contract: first-occurrence
    # relabelling, identical partition. Cluster labels may differ
    # between impls (e.g. R's clusters::igraph.reindex.membership
    # uses sort-by-key for the large-id branch), so the conformance
    # test compares by partition + canonical relabel.
    {
        "case": "reindex_membership_c_fast_path_dense",
        "origin": "C reference community_misc.c igraph_reindex_membership: "
        "fast-path branch (max_id < n) on already-dense input — identity output",
        "membership": [0, 1, 2, 0, 1, 2],
        "expected": {"membership": [0, 1, 2, 0, 1, 2], "new_to_old": [0, 1, 2]},
    },
    {
        "case": "reindex_membership_c_large_id_sparse",
        "origin": "C reference community_misc.c igraph_i_reindex_membership_large: "
        "sparse branch (max_id >> n) — sort-ascending then peel by group",
        "membership": [1000000, 7, 1000000, 7],
        "expected": {"membership": [0, 1, 0, 1], "new_to_old": [1000000, 7]},
    },
]

# ALGO-MST-001: minimum_spanning_tree. Upstream C
# tests/unit/minimum_spanning_tree.c builds an Erdős–Rényi graph with a
# fixed RNG seed (77685), which is not portable to Rust without porting
# the upstream RNG. We instead encode the three branches the C source
# exercises (BFS-unweighted spanning forest at spanning_trees.c:70;
# Prim at spanning_trees.c:176; Kruskal at spanning_trees.c:337) on
# tiny hand-derived graphs where the MST is uniquely determined by edge
# weights, then assert the matroid invariant (total weight + edge count)
# rather than exact edge IDs.
SPANNING_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "spanning_tree_c_k4_distinct_weights_automatic",
        # K4 with strictly increasing weights → MST = 3 lightest edges
        # 0,1,2 incident to vertex 0 (total = 6.0). Automatic dispatch
        # (weights provided ⇒ Kruskal per spanning_trees.c:461 dispatch).
        "origin": "constructed (mirrors spanning_trees.c igraph_minimum_spanning_tree "
        "AUTOMATIC dispatch with weights): K4 with edge weights [1..6]",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "graph_weights": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        "algo": "minimum_spanning_tree",
        "params": {"method": "automatic"},
        "expected": {"total_weight": 6.0, "edge_count": 3},
    },
    {
        "case": "spanning_tree_c_p4_unweighted_bfs",
        # Already-a-tree path — UNWEIGHTED dispatch returns the same
        # tree (spanning_trees.c:70 BFS branch).
        "origin": "constructed (mirrors spanning_trees.c "
        "igraph_i_minimum_spanning_tree_unweighted): P4 chain",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "minimum_spanning_tree",
        "params": {"method": "unweighted"},
        "expected": {"total_weight": 3.0, "edge_count": 3},
    },
    {
        "case": "spanning_tree_c_triangle_with_heavy_diagonal_prim",
        # Triangle with one heavy edge (5) and two light edges (1, 2) —
        # Prim must drop the 5-weight edge (spanning_trees.c:176 branch).
        "origin": "constructed (mirrors spanning_trees.c "
        "igraph_i_minimum_spanning_tree_prim): triangle (1, 2, 5)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (0, 2)], directed=False
        ),
        "graph_weights": [1.0, 2.0, 5.0],
        "algo": "minimum_spanning_tree",
        "params": {"method": "prim"},
        "expected": {"total_weight": 3.0, "edge_count": 2},
    },
]

# ALGO-GN-001: erdos_renyi_gnp / erdos_renyi_gnm. Mirrors the cases
# checked by `examples/simple/igraph_erdos_renyi_game.c` upstream:
# vcount must equal `n`, ecount must match the binomial expectation
# within a wide band for gnp and exactly for gnm. RNG state isn't
# portable from C to Rust, so we only assert structural invariants —
# the same shape the upstream example asserts via `igraph_vcount` and
# `igraph_ecount`.
ERDOS_RENYI_GNP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "erdos_renyi_gnp_c_undirected_no_loops_n30_p02",
        # G(30, 0.2). max_edges = 30·29/2 = 435. µ = 87, σ ≈ 8.34,
        # ±6σ band ≈ [37, 137] → use [35, 140].
        "origin": "constructed (mirrors examples/simple/"
        "igraph_erdos_renyi_game.c: IGRAPH_UNDIRECTED + no loops "
        "vcount/ecount sanity invariants)",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 30,
            "p": 0.2,
            "directed": False,
            "loops": False,
            "seed": 8_675_309,
        },
        "expected": {
            "vcount": 30,
            "ecount_min": 35,
            "ecount_max": 140,
            "directed": False,
        },
    },
    {
        "case": "erdos_renyi_gnp_c_directed_loops_n12_p04",
        # Directed with loops. max_edges = 12·12 = 144 (n² including
        # diagonal). µ = 57.6, σ ≈ 5.88, ±6σ band ≈ [22, 93] → [20, 95].
        "origin": "constructed (mirrors examples/simple/"
        "igraph_erdos_renyi_game.c: IGRAPH_DIRECTED + IGRAPH_LOOPS_SW)",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 12,
            "p": 0.4,
            "directed": True,
            "loops": True,
            "seed": 1_618_033,
        },
        "expected": {
            "vcount": 12,
            "ecount_min": 20,
            "ecount_max": 95,
            "directed": True,
        },
    },
    {
        "case": "erdos_renyi_gnp_c_undirected_loops_n6_p05",
        # Small undirected with self-loops. max_edges = 6·7/2 = 21
        # (triangular incl. diagonal). µ = 10.5, σ ≈ 2.29, ±6σ band
        # ≈ [-3, 24] → clamp to [0, 21].
        "origin": "constructed (mirrors examples/simple/"
        "igraph_erdos_renyi_game.c: IGRAPH_UNDIRECTED + IGRAPH_LOOPS_SW, "
        "small-n boundary)",
        "algo": "erdos_renyi_gnp",
        "params": {
            "n": 6,
            "p": 0.5,
            "directed": False,
            "loops": True,
            "seed": 271_828,
        },
        "expected": {
            "vcount": 6,
            "ecount_min": 0,
            "ecount_max": 21,
            "directed": False,
        },
    },
]

ERDOS_RENYI_GNM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "erdos_renyi_gnm_c_undirected_n20_m30",
        # G(20, 30) undirected, no loops. Sampling without replacement
        # → ecount exact.
        "origin": "constructed (mirrors examples/simple/"
        "igraph_erdos_renyi_game.c: GNM IGRAPH_UNDIRECTED, ecount==m)",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 20,
            "m": 30,
            "directed": False,
            "loops": False,
            "seed": 90_210,
        },
        "expected": {"vcount": 20, "ecount": 30, "directed": False},
    },
    {
        "case": "erdos_renyi_gnm_c_directed_loops_n6_m12",
        # Directed with self-loops allowed. max_edges = 36, picks 12.
        "origin": "constructed (mirrors examples/simple/"
        "igraph_erdos_renyi_game.c: GNM IGRAPH_DIRECTED + IGRAPH_LOOPS_SW)",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 6,
            "m": 12,
            "directed": True,
            "loops": True,
            "seed": 777,
        },
        "expected": {"vcount": 6, "ecount": 12, "directed": True},
    },
    {
        "case": "erdos_renyi_gnm_c_n0_m0_empty",
        # n=0 forces an empty graph regardless of seed — guards the
        # zero-vertex boundary that igraph C explicitly handles in
        # games/erdos_renyi.c.
        "origin": "constructed (mirrors igraph_erdos_renyi_game_gnm "
        "boundary: n=0 returns empty graph)",
        "algo": "erdos_renyi_gnm",
        "params": {
            "n": 0,
            "m": 0,
            "directed": False,
            "loops": False,
            "seed": 0,
        },
        "expected": {"vcount": 0, "ecount": 0, "directed": False},
    },
]

# ALGO-GN-002: barabasi_game_bag. Mirrors the BAG branch of
# `igraph_barabasi_game()` in games/barabasi.c:67-178. Like ER,
# generator RNG state isn't portable from C to Rust, so we capture the
# **structural invariants** that the upstream example
# `examples/simple/igraph_barabasi_game.c` and the unit test
# `tests/unit/igraph_barabasi_game.c` rely on:
#
#   * vcount: exact match with `params["n"]`.
#   * ecount: **exact** match with `(n - 1) * m` — the BAG variant is
#     deterministic in edge count when `m` is a scalar (see the
#     `outseq == NULL` branch at barabasi.c:113-117).
#   * directed: exact boolean match.
#   * ba_temporal_order: every edge `(src, dst)` satisfies `dst < src`
#     — preferential-attachment edges always point from the newly added
#     vertex to an earlier one (barabasi.c:158-170).
BARABASI_BAG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_game_bag_c_directed_n50_m2_no_outpref",
        "origin": "constructed (mirrors igraph_barabasi_game(n=50, "
        "m=2, outpref=0, algo=IGRAPH_BARABASI_BAG, directed=1)): "
        "edge count exact, BA temporal ordering",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 50,
            "m": 2,
            "outpref": False,
            "directed": True,
            "seed": 1_234_567,
        },
        "expected": {
            "vcount": 50,
            "ecount": 98,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "barabasi_game_bag_c_undirected_n25_m4",
        "origin": "constructed (mirrors igraph_barabasi_game(n=25, "
        "m=4, outpref=0, algo=IGRAPH_BARABASI_BAG, directed=0)): "
        "undirected forces outpref=true per barabasi.c:83-85",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 25,
            "m": 4,
            "outpref": False,
            "directed": False,
            "seed": 7_654_321,
        },
        "expected": {
            "vcount": 25,
            "ecount": 96,
            "directed": False,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "barabasi_game_bag_c_n1_singleton",
        "origin": "constructed (mirrors igraph_barabasi_game boundary "
        "n=1): single vertex, no edges regardless of m",
        "algo": "barabasi_game_bag",
        "params": {
            "n": 1,
            "m": 3,
            "outpref": False,
            "directed": True,
            "seed": 42,
        },
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
]

# ALGO-GN-003: growing_random_game. Mirrors `igraph_growing_random_game`
# in games/growing_random.c:55-105. Generator state isn't portable, so
# we capture structural invariants only:
#   * vcount: exact match with `params["n"]`.
#   * ecount: exact `(n - 1) * m`.
#   * directed: exact match.
#   * ba_temporal_order: only set when citation=true (every edge has
#     `dst < src` directed, `src != dst` undirected since storage
#     canonicalizes min/max).
GROWING_RANDOM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "growing_random_c_directed_citation_n40_m3",
        "origin": "constructed (mirrors igraph_growing_random_game(n=40, "
        "m=3, directed=1, citation=1)): citation edges always span "
        "(new -> earlier)",
        "algo": "growing_random_game",
        "params": {
            "n": 40,
            "m": 3,
            "directed": True,
            "citation": True,
            "seed": 9_876_543,
        },
        "expected": {
            "vcount": 40,
            "ecount": 117,
            "directed": True,
            "ba_temporal_order": True,
        },
    },
    {
        "case": "growing_random_c_undirected_free_n20_m2",
        "origin": "constructed (mirrors igraph_growing_random_game(n=20, "
        "m=2, directed=0, citation=0)): free-mode picks both endpoints",
        "algo": "growing_random_game",
        "params": {
            "n": 20,
            "m": 2,
            "directed": False,
            "citation": False,
            "seed": 1_357_911,
        },
        "expected": {
            "vcount": 20,
            "ecount": 38,
            "directed": False,
            "ba_temporal_order": False,
        },
    },
    {
        "case": "growing_random_c_n1_singleton",
        "origin": "constructed (mirrors igraph_growing_random_game boundary "
        "n=1): single vertex, no edges regardless of m",
        "algo": "growing_random_game",
        "params": {
            "n": 1,
            "m": 5,
            "directed": True,
            "citation": True,
            "seed": 24_680,
        },
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "ba_temporal_order": False,
        },
    },
]

# ALGO-GN-004: tree_game (LERW). Mirrors `igraph_tree_game` with
# `IGRAPH_RANDOM_TREE_LERW`. RNG state is not portable, so we encode
# structural invariants only — vcount, ecount=max(0, n-1), directed flag,
# and the spanning-tree property (acyclic + connected on the undirected
# projection, checked by union-find in the Rust harness).
TREE_LERW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "tree_lerw_c_undirected_n20",
        "origin": "constructed (mirrors igraph_tree_game(n=20, directed=false, "
        "method=IGRAPH_RANDOM_TREE_LERW)): small spanning tree",
        "algo": "tree_game_lerw",
        "params": {"n": 20, "directed": False, "seed": 1_111_111},
        "expected": {"vcount": 20, "ecount": 19, "directed": False, "is_tree": True},
    },
    {
        "case": "tree_lerw_c_directed_n40",
        "origin": "constructed (mirrors igraph_tree_game(n=40, directed=true, "
        "method=IGRAPH_RANDOM_TREE_LERW)): directed Wilson tree",
        "algo": "tree_game_lerw",
        "params": {"n": 40, "directed": True, "seed": 2_222_222},
        "expected": {"vcount": 40, "ecount": 39, "directed": True, "is_tree": True},
    },
    {
        "case": "tree_lerw_c_n2_single_edge",
        "origin": "constructed (mirrors igraph_tree_game boundary n=2): a "
        "single edge between vertices 0 and 1",
        "algo": "tree_game_lerw",
        "params": {"n": 2, "directed": False, "seed": 3_333_333},
        "expected": {"vcount": 2, "ecount": 1, "directed": False, "is_tree": True},
    },
]

# ALGO-GN-005: grg_game. Mirrors `igraph_grg_game`. RNG state is not
# portable, so we encode structural invariants only — vcount, undirected,
# simple (no self-loops, no multi-edges; checked by the harness via
# expected.is_simple), and a loose edge-density band (expected.ecount_min /
# ecount_max) anchored on the predicted Poisson mean n·(n-1)/2 · π·r²
# (plane interior) or `· min(π·r², 1)` (torus saturated).
GRG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "grg_c_plane_n50_r020",
        "origin": "constructed (mirrors igraph_grg_game(n=50, r=0.20, "
        "torus=false)): low-density disk-graph",
        "algo": "grg_game",
        # n=50, r=0.20: predicted = 50·49/2 · π·0.04 ≈ 154 edges. Wide
        # ±60 % band [60, 240] absorbs RNG variance.
        "params": {"n": 50, "radius": 0.20, "torus": False, "seed": 5_550_001},
        "expected": {
            "vcount": 50,
            "directed": False,
            "is_simple": True,
            "ecount_min": 60,
            "ecount_max": 240,
        },
    },
    {
        "case": "grg_c_torus_n80_r015",
        "origin": "constructed (mirrors igraph_grg_game(n=80, r=0.15, "
        "torus=true)): torus boundary",
        "algo": "grg_game",
        # n=80, r=0.15: predicted = 80·79/2 · π·0.0225 ≈ 223 edges.
        "params": {"n": 80, "radius": 0.15, "torus": True, "seed": 5_550_002},
        "expected": {
            "vcount": 80,
            "directed": False,
            "is_simple": True,
            "ecount_min": 90,
            "ecount_max": 360,
        },
    },
    {
        "case": "grg_c_zero_radius_n30",
        "origin": "constructed (mirrors igraph_grg_game boundary r=0): no edges",
        "algo": "grg_game",
        "params": {"n": 30, "radius": 0.0, "torus": False, "seed": 5_550_003},
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
]

# ALGO-GN-006: forest_fire_game. Mirrors `igraph_forest_fire_game` in
# games/forestfire.c:106-257 (Leskovec-Kleinberg-Faloutsos KDD'05,
# corrected variant). RNG state is not portable, so we capture
# structural invariants only — vcount, directed flag, is_simple (no
# self-loops, no duplicate directed edges, no parallels) and a loose
# ecount band: lower bound ≈ n-1 (one ambassador edge per new vertex),
# upper bound generous to absorb burn-tail variance.
# ALGO-FL-002: max_flow_value. Mirrors `igraph_maxflow_value` in
# references/igraph/src/flow/flow.c. The igraph C unit test
# tests/unit/igraph_maxflow.c covers two scenarios; we mirror the
# small undirected one (the other reads from a DIMACS file we don't
# bundle). The "no-capacity" variant supplements the C case with the
# same graph at unit capacity (max flow = bottleneck count of
# vertex-disjoint paths from 0 to 3 = 2).
MAXFLOW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "maxflow_c_undirected_4v_weighted",
        "origin": "tests/unit/igraph_maxflow.c:213-228 — undirected 4-vertex graph "
        "with edges (0-1,0-2,1-2,1-3,2-3) and capacities (4,2,10,2,2), "
        "source=0, target=3 → max flow = 4 (bottleneck = (1,3)+(2,3))",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "graph_weights": [4.0, 2.0, 10.0, 2.0, 2.0],
        "algo": "max_flow_value",
        "params": {"source": 0, "target": 3, "use_capacity": True},
        "expected": 4.0,
    },
    {
        "case": "maxflow_c_undirected_4v_unit",
        "origin": "tests/unit/igraph_maxflow.c structure (undirected 4-vertex "
        "graph) with unit capacities → 2 vertex-disjoint paths 0→1→3 and "
        "0→2→3, so unit max-flow value = 2",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "algo": "max_flow_value",
        "params": {"source": 0, "target": 3, "use_capacity": False},
        "expected": 2.0,
    },
]

# ALGO-FL-010: st_mincut_value. Mirrors `igraph_st_mincut_value` in
# references/igraph/src/flow/flow.c:1127 — a 5-line wrapper around
# `igraph_maxflow_value` justified by Ford-Fulkerson's max-flow /
# min-cut theorem (Ford-Fulkerson, 1956). The dedicated C unit test
# tests/unit/igraph_st_mincut_value.c:23-42 builds a 6-vertex directed
# graph with edges (0,1)(0,2)(1,2)(1,3)(2,4)(3,4)(3,5)(4,5) and
# capacities [5,2,2,3,4,1,2,5], asserts mincut(0→5) == 7. We mirror
# that fixture verbatim plus a unit-capacity variant of the small
# undirected 4-vertex graph for cross-source uniformity.
ST_MINCUT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_mincut_c_directed_6v_weighted",
        "origin": "tests/unit/igraph_st_mincut_value.c:23-42 — 6-vertex directed "
        "graph with edges (0,1)(0,2)(1,2)(1,3)(2,4)(3,4)(3,5)(4,5) and "
        "capacities [5,2,2,3,4,1,2,5], source=0, target=5 → "
        "st_mincut_value == 7",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 4), (3, 4), (3, 5), (4, 5)],
            directed=True,
        ),
        "graph_weights": [5.0, 2.0, 2.0, 3.0, 4.0, 1.0, 2.0, 5.0],
        "algo": "st_mincut_value",
        "params": {"source": 0, "target": 5, "use_capacity": True},
        "expected": 7.0,
    },
    {
        "case": "st_mincut_c_undirected_4v_unit",
        "origin": "tests/unit/igraph_maxflow.c structure (undirected 4-vertex "
        "graph) with unit capacities → 2 vertex-disjoint paths from 0 to "
        "3, so unit st_mincut_value = 2 (duality with max-flow)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "algo": "st_mincut_value",
        "params": {"source": 0, "target": 3, "use_capacity": False},
        "expected": 2.0,
    },
]

# ALGO-FL-011: st_edge_connectivity. Mirrors `igraph_st_edge_connectivity`
# in references/igraph/src/flow/flow.c:2219 — a 15-line wrapper around
# `igraph_maxflow_value` with NULL capacity (unit caps), cast to integer.
# The dedicated C unit test tests/unit/igraph_st_edge_connectivity.c:23-38
# builds a 6-vertex directed graph with edges
# (0,1)(0,2)(1,2)(1,3)(2,4)(3,4)(3,5)(4,5), asserts ec(0→5) == 2.
ST_EDGE_CONN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_edge_conn_c_directed_6v",
        "origin": "tests/unit/igraph_st_edge_connectivity.c:23-38 — 6-vertex "
        "directed graph with edges (0,1)(0,2)(1,2)(1,3)(2,4)(3,4)(3,5)"
        "(4,5), source=0, target=5 → st_edge_connectivity == 2",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 4), (3, 4), (3, 5), (4, 5)],
            directed=True,
        ),
        "algo": "st_edge_connectivity",
        "params": {"source": 0, "target": 5},
        "expected": 2,
    },
    {
        "case": "st_edge_conn_c_undirected_path_4v",
        "origin": "structural: 0—1—2—3 undirected path, every edge is a "
        "bottleneck → st_edge_connectivity(0, 3) == 1",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "st_edge_connectivity",
        "params": {"source": 0, "target": 3},
        "expected": 1,
    },
]

# ALGO-FL-012: edge_disjoint_paths. Mirrors `igraph_edge_disjoint_paths`
# in references/igraph/src/flow/flow.c:2326 — a 15-line wrapper around
# `igraph_maxflow_value` with NULL capacity (unit caps), cast to integer.
# By Menger's theorem the max number of edge-disjoint s→t paths equals
# the unit-capacity max-flow. The dedicated C unit test
# tests/unit/igraph_edge_disjoint_paths.c:23-46 builds a 6-vertex directed
# graph with edges (0,1)(0,2)(1,2)(1,3)(2,4)(3,4)(3,5)(4,5)(3,3) (note the
# self-loop at vertex 3), asserts ep(0→5)=2, ep(0→3)=1, ep(3→0)=0,
# ep(3→5)=2; then converts to undirected and asserts ep(4→3)=3.
ED_PATHS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_disjoint_paths_c_directed_0_to_5",
        "origin": "tests/unit/igraph_edge_disjoint_paths.c:31-32 — 6-vertex "
        "directed graph with edges (0,1)(0,2)(1,2)(1,3)(2,4)(3,4)(3,5)(4,5)"
        "(3,3 self-loop), source=0, target=5 → edge_disjoint_paths == 2",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (3, 3),
            ],
            directed=True,
        ),
        "algo": "edge_disjoint_paths",
        "params": {"source": 0, "target": 5},
        "expected": 2,
    },
    {
        "case": "edge_disjoint_paths_c_directed_0_to_3",
        "origin": "tests/unit/igraph_edge_disjoint_paths.c:34-35 — same "
        "6-vertex directed fixture, source=0, target=3 → "
        "edge_disjoint_paths == 1 (bottleneck via vertex 1)",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (3, 3),
            ],
            directed=True,
        ),
        "algo": "edge_disjoint_paths",
        "params": {"source": 0, "target": 3},
        "expected": 1,
    },
    {
        "case": "edge_disjoint_paths_c_directed_3_to_0",
        "origin": "tests/unit/igraph_edge_disjoint_paths.c:37-38 — same "
        "6-vertex directed fixture, source=3, target=0 → "
        "edge_disjoint_paths == 0 (no reverse path)",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (3, 3),
            ],
            directed=True,
        ),
        "algo": "edge_disjoint_paths",
        "params": {"source": 3, "target": 0},
        "expected": 0,
    },
    {
        "case": "edge_disjoint_paths_c_directed_3_to_5",
        "origin": "tests/unit/igraph_edge_disjoint_paths.c:40-41 — same "
        "6-vertex directed fixture, source=3, target=5 → "
        "edge_disjoint_paths == 2 (direct (3,5) + (3,4)→(4,5))",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (3, 3),
            ],
            directed=True,
        ),
        "algo": "edge_disjoint_paths",
        "params": {"source": 3, "target": 5},
        "expected": 2,
    },
    {
        "case": "edge_disjoint_paths_c_undirected_4_to_3",
        "origin": "tests/unit/igraph_edge_disjoint_paths.c:43-46 — same "
        "fixture after igraph_to_undirected (each arc → one edge), "
        "source=4, target=3 → edge_disjoint_paths == 3 (direct edge + "
        "via 2→1 + via 5)",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (3, 3),
            ],
            directed=False,
        ),
        "algo": "edge_disjoint_paths",
        "params": {"source": 4, "target": 3},
        "expected": 3,
    },
]

# ALGO-FL-013: st_vertex_connectivity. Mirrors `igraph_st_vertex_connectivity`
# in references/igraph/src/flow/flow.c:1922 — uses the vertex-splitting
# reduction (`igraph_i_split_vertices` from flow_conversion.c:61) and a
# unit-cap max-flow on the split graph. The dedicated C unit test
# tests/unit/igraph_st_vertex_connectivity.c:32-66 has nine print-cases;
# the three trailing CHECK_ERROR cases at lines 70-83 verify error paths
# (source==target, n==0, n==1) and are exercised in the Rust unit-test
# module directly rather than via the JSON conformance harness.
ST_VCONN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_vconn_c_two_unconnected_error",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:32-34 — 2v "
        "undirected with no edges, s=0, t=1, mode=ERROR → 0 (no direct "
        "edge, max-flow on split graph = 0)",
        "graph_factory": lambda: ig.Graph(n=2, edges=[], directed=False),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "error"},
        "expected": 0,
    },
    {
        "case": "st_vconn_c_two_connected_negative",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:36-38 — 2v "
        "undirected with one edge (0,1), mode=NEGATIVE → -1",
        "graph_factory": lambda: ig.Graph(n=2, edges=[(0, 1)], directed=False),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "negative"},
        "expected": -1,
    },
    {
        "case": "st_vconn_c_two_connected_number_of_nodes",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:40-42 — same "
        "2v undirected fixture, mode=NUMBER_OF_NODES → 2",
        "graph_factory": lambda: ig.Graph(n=2, edges=[(0, 1)], directed=False),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "number_of_nodes"},
        "expected": 2,
    },
    {
        "case": "st_vconn_c_three_parallel_undirected_ignore",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:44-46 — 2v "
        "undirected with 3 parallel edges (0,1)×3, mode=IGNORE → 0 "
        "(direct arcs subtracted)",
        "graph_factory": lambda: ig.Graph(
            n=2, edges=[(0, 1), (0, 1), (0, 1)], directed=False
        ),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "ignore"},
        "expected": 0,
    },
    {
        "case": "st_vconn_c_mixed_parallel_undirected_ignore",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:48-50 — 2v "
        "undirected with (0,1)×3 + (1,0)×2 (printed as 'directed' but the "
        "C `igraph_small` call is IGRAPH_UNDIRECTED), mode=IGNORE → 0",
        "graph_factory": lambda: ig.Graph(
            n=2,
            edges=[(0, 1), (0, 1), (0, 1), (1, 0), (1, 0)],
            directed=False,
        ),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "ignore"},
        "expected": 0,
    },
    {
        "case": "st_vconn_c_line_graph_6v_error",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:52-54 — 6v "
        "undirected path 0-1-2-3-4-5, s=0, t=5, mode=ERROR → 1 "
        "(any internal vertex cuts)",
        "graph_factory": lambda: ig.Graph(
            n=6, edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)], directed=False
        ),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 5, "mode": "error"},
        "expected": 1,
    },
    {
        "case": "st_vconn_c_full_graph_6v_undirected_ignore",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:56-58 — K_6 "
        "undirected, s=0, t=1, mode=IGNORE → 4 (must remove all "
        "internal vertices)",
        "graph_factory": lambda: ig.Graph.Full(n=6, directed=False, loops=False),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "ignore"},
        "expected": 4,
    },
    {
        "case": "st_vconn_c_full_graph_6v_directed_ignore",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:60-62 — K_6 "
        "directed (both arcs for every pair), s=0, t=1, mode=IGNORE → 4",
        "graph_factory": lambda: ig.Graph.Full(n=6, directed=True, loops=False),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "ignore"},
        "expected": 4,
    },
    {
        "case": "st_vconn_c_three_vertex_bottleneck_error",
        "origin": "tests/unit/igraph_st_vertex_connectivity.c:64-66 — 3v "
        "undirected (0,1)×2 + (1,2)×4, s=0, t=2, mode=ERROR → 1 "
        "(vertex 1 is the only path)",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (0, 1), (1, 2), (1, 2), (1, 2), (1, 2)],
            directed=False,
        ),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 2, "mode": "error"},
        "expected": 1,
    },
]

# ALGO-FL-014: vertex_disjoint_paths. Mirrors `igraph_vertex_disjoint_paths`
# in references/igraph/src/flow/flow.c:2374 — calls
# `igraph_i_st_vertex_connectivity_{directed,undirected}` with
# IGRAPH_VCONN_NEI_IGNORE and adds the direct-edge count back. The dedicated
# C unit test tests/unit/igraph_vertex_disjoint_paths.c:23-52 has five
# IGRAPH_ASSERT cases (3 directed + 2 undirected on the same 7v multigraph).
VDP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "vdp_c_directed_0_to_5",
        "origin": "tests/unit/igraph_vertex_disjoint_paths.c:32-33 — 7v "
        "directed multigraph with direct edge 0→5, self-loop at 3, mutual "
        "arcs (1,3)+(3,1); vdp(0,5)=3 (one direct + two via interior).",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (0, 5), (3, 3),
                (5, 2), (1, 3), (3, 1),
            ],
            directed=True,
        ),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 0, "target": 5},
        "expected": 3,
    },
    {
        "case": "vdp_c_directed_1_to_3",
        "origin": "tests/unit/igraph_vertex_disjoint_paths.c:35-36 — same "
        "7v directed multigraph; vdp(1,3)=2 — the two parallel mutual "
        "(1,3) arcs count once for the direct-edge bonus, plus one "
        "internal path via vertex 2 / 4.",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (0, 5), (3, 3),
                (5, 2), (1, 3), (3, 1),
            ],
            directed=True,
        ),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 1, "target": 3},
        "expected": 2,
    },
    {
        "case": "vdp_c_directed_4_to_0",
        "origin": "tests/unit/igraph_vertex_disjoint_paths.c:38-39 — same "
        "7v directed multigraph; vdp(4,0)=0 (no path from 4 back to 0 in "
        "directed orientation).",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (0, 5), (3, 3),
                (5, 2), (1, 3), (3, 1),
            ],
            directed=True,
        ),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 4, "target": 0},
        "expected": 0,
    },
    {
        "case": "vdp_c_undirected_4_to_0",
        "origin": "tests/unit/igraph_vertex_disjoint_paths.c:43-44 — same "
        "fixture after igraph_to_undirected(EACH); vdp(4,0)=3.",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (0, 5), (3, 3),
                (5, 2), (1, 3), (3, 1),
            ],
            directed=False,
        ),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 4, "target": 0},
        "expected": 3,
    },
    {
        "case": "vdp_c_undirected_1_to_3",
        "origin": "tests/unit/igraph_vertex_disjoint_paths.c:46-47 — same "
        "undirected fixture; vdp(1,3)=5 (three parallel direct edges plus "
        "two interior-disjoint paths).",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (0, 5), (3, 3),
                (5, 2), (1, 3), (3, 1),
            ],
            directed=False,
        ),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 1, "target": 3},
        "expected": 5,
    },
]

# ALGO-FL-015: vertex_connectivity (global cohesion). Mirrors
# `igraph_vertex_connectivity` in references/igraph/src/flow/flow.c:2158
# and its alias `igraph_cohesion` in flow.c:2470. The C unit test
# tests/unit/igraph_cohesion.c:29-44 has two IGRAPH_ASSERT cases on the
# same 7v edge list (directed → vc=1, undirected → vc=2). We also pin a
# few short-circuit cases (empty / disconnected / tree / complete /
# ring) since the C dispatcher in flow.c:2158-2192 has four branches we
# want full per-branch coverage on.
VCONN_GLOBAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "vconn_c_directed_7v_equals_one",
        "origin": "tests/unit/igraph_cohesion.c:29-34 — 7v directed "
        "graph (edges 0-1 0-2 1-2 1-3 2-4 3-4 3-5 4-5 1-6 6-3 5-0); "
        "igraph_cohesion returns 1.",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (1, 6), (6, 3), (5, 0),
            ],
            directed=True,
        ),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 1,
    },
    {
        "case": "vconn_c_undirected_7v_equals_two",
        "origin": "tests/unit/igraph_cohesion.c:38-43 — 7v undirected "
        "graph (edges 0-1 0-2 1-2 1-3 2-4 3-4 3-5 4-5 1-6 6-3); "
        "igraph_cohesion returns 2.",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (1, 6), (6, 3),
            ],
            directed=False,
        ),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 2,
    },
    {
        "case": "vconn_c_empty_returns_zero",
        "origin": "C dispatcher flow.c:2084-2087 — empty graph short "
        "circuit returns 0.",
        "graph_factory": lambda: ig.Graph(n=0, edges=[], directed=False),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 0,
    },
    {
        "case": "vconn_c_two_isolated_components_returns_zero",
        "origin": "C dispatcher flow.c:2090-2093 — disconnected "
        "(strongly for directed, weakly for undirected) short-circuit "
        "returns 0; two disjoint undirected edges 0-1 and 2-3.",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (2, 3)],
            directed=False,
        ),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 0,
    },
    {
        "case": "vconn_c_complete_undirected_6v_returns_five",
        "origin": "C dispatcher flow.c:2168-2180 — complete-graph "
        "short-circuit returns vcount-1 = 5 for K_6.",
        "graph_factory": lambda: ig.Graph.Full(6, directed=False, loops=False),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 5,
    },
]

# ALGO-FL-016: edge_connectivity (global adhesion). Mirrors
# `igraph_edge_connectivity` in references/igraph/src/flow/flow.c:2270
# and its alias `igraph_adhesion` at flow.c:2433. The dedicated C unit
# test tests/unit/igraph_edge_connectivity.c covers both the cheap
# short-circuits (singleton/disconnected/min-deg=1) and the fixed-vertex
# st_edge_connectivity loop reached via the no-shortcut path. We pin
# five branches: the two 7v fixtures shared with VCONN_GLOBAL (whose
# edge connectivity is computable from the same edge list — directed
# has a vertex with in/out=1 so cheap=1, undirected runs the full loop
# returning 2), the empty short-circuit, the disconnected short-circuit,
# and K_5 which exercises the no-shortcut fixed-vertex loop.
ECONN_GLOBAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "econn_c_directed_7v_equals_one",
        "origin": "C dispatcher flow.c:2270 + flow.c:2076 — 7v directed "
        "(same edge list as vconn_c_directed_7v) hits the min(in,out)=1 "
        "short-circuit (vertex 6 has out=1,in=1) ⇒ edge_connectivity=1.",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (1, 6), (6, 3), (5, 0),
            ],
            directed=True,
        ),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 1,
    },
    {
        "case": "econn_c_undirected_7v_equals_two",
        "origin": "C dispatcher flow.c:2270 + fixed-vertex loop "
        "flow.c:1706-1723 — 7v undirected (same edge list as "
        "vconn_c_undirected_7v); min-degree=2 so cheap checks pass; "
        "fixed-vertex loop isolates vertex 0 by removing {(0,1),(0,2)} "
        "⇒ edge_connectivity=2.",
        "graph_factory": lambda: ig.Graph(
            n=7,
            edges=[
                (0, 1), (0, 2), (1, 2), (1, 3), (2, 4),
                (3, 4), (3, 5), (4, 5), (1, 6), (6, 3),
            ],
            directed=False,
        ),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 2,
    },
    {
        "case": "econn_c_empty_returns_zero",
        "origin": "C dispatcher flow.c:2281-2284 — singleton/empty "
        "graph short circuit returns 0.",
        "graph_factory": lambda: ig.Graph(n=0, edges=[], directed=False),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 0,
    },
    {
        "case": "econn_c_two_isolated_components_returns_zero",
        "origin": "C dispatcher flow.c:2287-2289 → flow.c:2090-2093 — "
        "disconnected short-circuit returns 0 for two disjoint "
        "undirected edges 0-1 and 2-3.",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (2, 3)],
            directed=False,
        ),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 0,
    },
    {
        "case": "econn_c_complete_undirected_5v_returns_four",
        "origin": "C dispatcher flow.c:2287-2295 — K_5 undirected "
        "(min-degree=4, no cheap short-circuit) runs the fixed-vertex "
        "loop ⇒ edge_connectivity = 4 (n - 1 for simple K_n).",
        "graph_factory": lambda: ig.Graph.Full(5, directed=False, loops=False),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 4,
    },
]

# ALGO-FL-017: mincut_value (global minimum-cut value, weighted
# generalisation of FL-016). Mirrors `igraph_mincut_value` at
# `references/igraph/src/flow/flow.c:1692`. The dedicated C unit test
# tests/unit/igraph_mincut.c builds a small directed graph with mixed
# capacities. We pin (i) unit-cap parity with edge_connectivity, (ii)
# weighted fixtures exercising the fixed-vertex loop, (iii) the
# `vcount ≤ 1` IGRAPH_INFINITY corner case, (iv) directed unit-cap
# both-directions iteration.
MINCUT_VALUE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mincut_c_directed_3cycle_unit_caps_returns_one",
        "origin": "C dispatcher flow.c:1706-1723 — directed 3-cycle "
        "0→1→2→0 with unit capacities; every arc is a directed bridge "
        "⇒ mincut_value = 1.0.",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "algo": "mincut_value",
        "params": {"capacity": None},
        "expected": 1.0,
    },
    {
        "case": "mincut_c_directed_3cycle_weighted",
        "origin": "C dispatcher flow.c:1706-1723 — directed 3-cycle "
        "with weights [3, 1, 2]; bottleneck arc 1→2 weight 1 minimises "
        "every 0→v / v→0 cut ⇒ mincut_value = 1.0.",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "graph_weights": [3.0, 1.0, 2.0],
        "algo": "mincut_value",
        "params": {"capacity": [3.0, 1.0, 2.0]},
        "expected": 1.0,
    },
    {
        "case": "mincut_c_undirected_ring5_unit_caps_returns_two",
        "origin": "C dispatcher flow.c:1702 — undirected ring C_5 unit "
        "capacities; igraph_i_mincut_value_undirected (Stoer-Wagner in "
        "C; fixed-vertex loop here) ⇒ mincut_value = 2.0.",
        "graph_factory": lambda: ig.Graph.Ring(5, directed=False, circular=True),
        "algo": "mincut_value",
        "params": {"capacity": None},
        "expected": 2.0,
    },
    {
        "case": "mincut_c_two_isolated_edges_returns_zero",
        "origin": "C dispatcher: undirected, two isolated edges → "
        "disconnected ⇒ max_flow(0, v) = 0 for any v in the other "
        "component ⇒ mincut_value = 0.0.",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (2, 3)], directed=False
        ),
        "algo": "mincut_value",
        "params": {"capacity": None},
        "expected": 0.0,
    },
    {
        "case": "mincut_c_complete_undirected_4v_unit_caps_returns_three",
        "origin": "C dispatcher: K_4 undirected unit caps — every "
        "single-vertex isolation cut has weight 3 ⇒ mincut_value = 3.0 "
        "(matches n - 1 for simple K_n at unit caps).",
        "graph_factory": lambda: ig.Graph.Full(4, directed=False, loops=False),
        "algo": "mincut_value",
        "params": {"capacity": None},
        "expected": 3.0,
    },
]

# ALGO-FL-018: st_mincut (full s-t minimum-cut partition). Mirrors
# `igraph_st_mincut` at `references/igraph/src/flow/flow.c:1140` —
# a 47-line wrapper around `igraph_maxflow` that asks for the cut
# edge list and source / sink partitions in addition to the value.
# `expected` is a JSON object: `value` is always required; `cut`,
# `partition`, `partition2` are optional and only pinned when the
# minimum cut is unique (multiple optimal cuts may exist for the same
# value, so we don't over-constrain).
ST_MINCUT_PARTITION_MANIFEST: List[Dict[str, Any]] = [
    {
        # Replicates tests/unit/igraph_st_mincut.c verbatim
        # (no-capacity branch, lines 45-50 of the C test).
        "case": "st_mincut_c_directed_5v_unit_caps",
        "origin": "tests/unit/igraph_st_mincut.c:40-50 — 5-vertex directed "
        "graph (0,1)(1,2)(1,3)(2,4)(3,4), source=0 target=4, unit caps. "
        "Reference output cut=(0) partition=(0) partition2=(1,2,3,4); "
        "value = 1.0 since edge (0,1) is the unique source bridge.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (1, 3), (2, 4), (3, 4)], directed=True
        ),
        "algo": "st_mincut",
        "params": {"source": 0, "target": 4, "capacity": None},
        "expected": {
            "value": 1.0,
            "cut": [0],
            "partition": [0],
            "partition2": [1, 2, 3, 4],
        },
    },
    {
        # Replicates tests/unit/igraph_st_mincut.c verbatim
        # (weighted branch, lines 52-58 of the C test).
        "case": "st_mincut_c_directed_5v_weighted",
        "origin": "tests/unit/igraph_st_mincut.c:52-58 — same 5-vertex "
        "directed graph with capacities [8,2,3,3,2]; reference cut=(1,4) "
        "partition=(0,1,3) partition2=(2,4); value = 2.0 + 2.0 = 4.0.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (1, 3), (2, 4), (3, 4)], directed=True
        ),
        "graph_weights": [8.0, 2.0, 3.0, 3.0, 2.0],
        "algo": "st_mincut",
        "params": {
            "source": 0,
            "target": 4,
            "capacity": [8.0, 2.0, 3.0, 3.0, 2.0],
        },
        "expected": {
            "value": 4.0,
            "cut": [1, 4],
            "partition": [0, 1, 3],
            "partition2": [2, 4],
        },
    },
    {
        # CLRS 26.1-1: max flow = 23. Multiple min cuts may exist with
        # value 23, so pin value only — the runner additionally checks
        # the structural invariants (partition covers V, source in
        # partition, target in partition2, sum cut caps == value, cut
        # disconnects s from t).
        "case": "st_mincut_c_clrs_textbook_value_only",
        "origin": "CLRS 26.1-1 classic max-flow network mirrored from "
        "igraph_maxflow.c tests; 6-vertex directed with edges "
        "(0,1)(0,2)(1,3)(2,1)(2,4)(3,2)(3,5)(4,3)(4,5) caps "
        "[16,13,12,4,14,9,20,7,4]; max flow = min cut = 23.",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[
                (0, 1),
                (0, 2),
                (1, 3),
                (2, 1),
                (2, 4),
                (3, 2),
                (3, 5),
                (4, 3),
                (4, 5),
            ],
            directed=True,
        ),
        "graph_weights": [16.0, 13.0, 12.0, 4.0, 14.0, 9.0, 20.0, 7.0, 4.0],
        "algo": "st_mincut",
        "params": {
            "source": 0,
            "target": 5,
            "capacity": [16.0, 13.0, 12.0, 4.0, 14.0, 9.0, 20.0, 7.0, 4.0],
        },
        "expected": {"value": 23.0},
    },
    {
        # Undirected 4-vertex from igraph_maxflow.c with explicit caps;
        # unique min cut crosses at (1,3) and (2,3) with caps 2 + 2 = 4.
        # Pin value + partition + partition2 + cut.
        "case": "st_mincut_c_undirected_4v_weighted",
        "origin": "Adapted from igraph_maxflow.c 4-vertex undirected "
        "reference: edges (0,1)(0,2)(1,2)(1,3)(2,3) with caps "
        "[4,2,10,2,2]; min cut between {0,1,2} and {3} = 2+2 = 4.",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)], directed=False
        ),
        "graph_weights": [4.0, 2.0, 10.0, 2.0, 2.0],
        "algo": "st_mincut",
        "params": {
            "source": 0,
            "target": 3,
            "capacity": [4.0, 2.0, 10.0, 2.0, 2.0],
        },
        "expected": {
            "value": 4.0,
            "cut": [3, 4],
            "partition": [0, 1, 2],
            "partition2": [3],
        },
    },
]

# ALGO-FL-020: gomory_hu_tree. Mirrors `igraph_gomory_hu_tree` at
# `references/igraph/src/flow/flow.c:2479-2616` and the C unit fixtures
# in `references/igraph/tests/unit/igraph_gomory_hu_tree.c`. The tree
# itself is not unique (Gusfield depends on iteration order), so the
# manifest pins only *shape invariants* and a `flows_min` floor; the
# Rust runner additionally verifies the Gomory-Hu property by
# recomputing `max_flow_value` for every pair and asserting equality
# with the min-edge-weight along the unique tree path between them.
GOMORY_HU_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "gomory_hu_c_empty",
        "origin": "tests/unit/igraph_gomory_hu_tree.c:170-176 — "
        "empty undirected graph (n=0); tree vcount=0, flows empty.",
        "graph_factory": lambda: ig.Graph(n=0, edges=[], directed=False),
        "algo": "gomory_hu_tree",
        "params": {"capacity": None},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "flows_len": 0,
            "is_directed": False,
        },
    },
    {
        "case": "gomory_hu_c_6v_weighted",
        "origin": "tests/unit/igraph_gomory_hu_tree.c:178-191 — "
        "6-vertex undirected (0-1)(0-2)(1-2)(1-3)(1-4)(2-4)(3-4)(3-5)"
        "(4-5) caps [1,7,1,3,2,4,1,6,2]. validate_tree compares each "
        "pair's tree-path min weight against max_flow_value.",
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
        "case": "gomory_hu_c_k4_unit_caps",
        "origin": "tests/unit/igraph_gomory_hu_tree.c:199-204 — "
        "K_4 undirected unit caps (issue #1810 regression). Every "
        "pair has max-flow 3 (degree); every tree edge weight is 3.",
        "graph_factory": lambda: ig.Graph.Full(4, directed=False, loops=False),
        "algo": "gomory_hu_tree",
        "params": {"capacity": None},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "flows_len": 3,
            "flows_min": 3.0,
            "is_directed": False,
        },
    },
    {
        "case": "gomory_hu_c_6v_directed_rejects",
        "origin": "tests/unit/igraph_gomory_hu_tree.c:206-212 — "
        "same 6v edge set directed=true returns IGRAPH_EINVAL; "
        "Gomory-Hu is defined only for undirected graphs.",
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
            directed=True,
        ),
        "graph_weights": [1.0, 7.0, 1.0, 3.0, 2.0, 4.0, 1.0, 6.0, 2.0],
        "algo": "gomory_hu_tree",
        "params": {"capacity": [1.0, 7.0, 1.0, 3.0, 2.0, 4.0, 1.0, 6.0, 2.0]},
        "expected": {"raises": True},
    },
]

# ALGO-FL-030: dominator_tree (Lengauer-Tarjan). Three fixtures lifted
# directly from `references/igraph/tests/unit/igraph_dominator_tree.c`
# (and its `.out` reference) plus one negative case. The reference idom
# vectors use the same `-1` = root / `-2` = unreachable sentinels as the
# Rust port.
DOMINATOR_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dominator_c_13v_classical_out",
        "origin": "tests/unit/igraph_dominator_tree.c:28-56 — "
        "13-vertex directed Lengauer-Tarjan example; root=0, mode=OUT.",
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
        "case": "dominator_c_13v_reversed_in",
        "origin": "tests/unit/igraph_dominator_tree.c:65-89 — "
        "same 13v flowgraph with every edge reversed, mode=IN.",
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
        "case": "dominator_c_20v_unreachable_out",
        "origin": "tests/unit/igraph_dominator_tree.c:101-121 — "
        "20-vertex graph with disconnected component {5,6,7,16..19}; "
        "mode=OUT, root=0. idom uses -2 for unreachable.",
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
        "case": "dominator_c_undirected_rejects",
        "origin": "constructed — igraph_dominator_tree returns "
        "IGRAPH_EINVAL on undirected input (the algorithm is defined "
        "only for directed flowgraphs).",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "out"},
        "expected": {"raises": True},
    },
]

FOREST_FIRE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "forest_fire_c_directed_n50_fw02_bw05_ambs2",
        "origin": "constructed (mirrors igraph_forest_fire_game(n=50, "
        "fw_prob=0.2, bw_factor=0.5, ambs=2, directed=true)): low-burn "
        "directed graph",
        "algo": "forest_fire_game",
        "params": {
            "n": 50,
            "fw_prob": 0.2,
            "bw_factor": 0.5,
            "ambs": 2,
            "directed": True,
            "seed": 4_440_001,
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
        "case": "forest_fire_c_undirected_n80_fw01_bw03_ambs1",
        "origin": "constructed (mirrors igraph_forest_fire_game(n=80, "
        "fw_prob=0.1, bw_factor=0.3, ambs=1, directed=false)): cool "
        "burn with single ambassador",
        "algo": "forest_fire_game",
        "params": {
            "n": 80,
            "fw_prob": 0.1,
            "bw_factor": 0.3,
            "ambs": 1,
            "directed": False,
            "seed": 4_440_002,
        },
        "expected": {
            "vcount": 80,
            "directed": False,
            "is_simple": True,
            "ecount_min": 79,
            "ecount_max": 8000,
        },
    },
    {
        "case": "forest_fire_c_ambs0_edgeless_n40",
        "origin": "constructed (mirrors igraph_forest_fire_game boundary "
        "ambs=0): edgeless graph regardless of fw_prob/bw_factor",
        "algo": "forest_fire_game",
        "params": {
            "n": 40,
            "fw_prob": 0.2,
            "bw_factor": 0.5,
            "ambs": 0,
            "directed": True,
            "seed": 4_440_003,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
]

# ALGO-GN-030: bipartite_game_gnp / bipartite_game_gnm. Mirrors
# `igraph_bipartite_game_gnp` and `igraph_bipartite_game_gnm` in
# misc/bipartite.c. RNG state is not portable across C/py/R bindings,
# so we capture structural invariants only — vcount==n1+n2, exact
# types partition (n1 false then n2 true), simple, every edge crosses
# the partition, gnm: ecount==m exactly, gnp: ecount in band around
# E[m]=p*max_edges with conservative ±4σ window.
BIPARTITE_GAME_GNP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bipartite_gnp_c_undirected_n10_n8_p03_all",
        "origin": "tests/unit/igraph_bipartite_game.c — gnp small "
        "undirected case (n1=10, n2=8, p=0.3, mode=all)",
        "algo": "bipartite_game_gnp",
        "params": {
            "n1": 10,
            "n2": 8,
            "p": 0.3,
            "directed": False,
            "mode": "all",
            "seed": 5_550_001,
        },
        "expected": {
            "vcount": 18,
            "n1": 10,
            "n2": 8,
            "directed": False,
            "is_simple": True,
            "ecount_min": 6,
            "ecount_max": 60,
            "bipartite_partitions": True,
        },
    },
    {
        "case": "bipartite_gnp_c_directed_n8_n6_p04_out",
        "origin": "tests/unit/igraph_bipartite_game.c — gnp directed "
        "out (n1=8, n2=6, p=0.4, mode=out, bottom→top arcs only)",
        "algo": "bipartite_game_gnp",
        "params": {
            "n1": 8,
            "n2": 6,
            "p": 0.4,
            "directed": True,
            "mode": "out",
            "seed": 5_550_003,
        },
        "expected": {
            "vcount": 14,
            "n1": 8,
            "n2": 6,
            "directed": True,
            "is_simple": True,
            "ecount_min": 6,
            "ecount_max": 48,
            "bipartite_partitions": True,
            "edges_bottom_to_top": True,
        },
    },
    {
        "case": "bipartite_gnp_c_undirected_n5_n4_p1_complete",
        "origin": "tests/unit/igraph_bipartite_game.c — gnp p=1 "
        "boundary; mode=all undirected yields complete K_{5,4}",
        "algo": "bipartite_game_gnp",
        "params": {
            "n1": 5,
            "n2": 4,
            "p": 1.0,
            "directed": False,
            "mode": "all",
            "seed": 5_550_004,
        },
        "expected": {
            "vcount": 9,
            "n1": 5,
            "n2": 4,
            "directed": False,
            "is_simple": True,
            "ecount_min": 20,
            "ecount_max": 20,
            "bipartite_partitions": True,
        },
    },
]

BIPARTITE_GAME_GNM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bipartite_gnm_c_undirected_n20_n15_m50_all",
        "origin": "tests/unit/igraph_bipartite_game.c — gnm exact "
        "count (n1=20, n2=15, m=50, mode=all, undirected)",
        "algo": "bipartite_game_gnm",
        "params": {
            "n1": 20,
            "n2": 15,
            "m": 50,
            "directed": False,
            "mode": "all",
            "seed": 5_550_002,
        },
        "expected": {
            "vcount": 35,
            "n1": 20,
            "n2": 15,
            "directed": False,
            "is_simple": True,
            "ecount_min": 50,
            "ecount_max": 50,
            "bipartite_partitions": True,
        },
    },
    {
        "case": "bipartite_gnm_c_directed_n6_n4_m12_in",
        "origin": "tests/unit/igraph_bipartite_game.c — gnm directed "
        "in (n1=6, n2=4, m=12, mode=in, top→bottom arcs only)",
        "algo": "bipartite_game_gnm",
        "params": {
            "n1": 6,
            "n2": 4,
            "m": 12,
            "directed": True,
            "mode": "in",
            "seed": 5_550_005,
        },
        "expected": {
            "vcount": 10,
            "n1": 6,
            "n2": 4,
            "directed": True,
            "is_simple": True,
            "ecount_min": 12,
            "ecount_max": 12,
            "bipartite_partitions": True,
            "edges_top_to_bottom": True,
        },
    },
    {
        "case": "bipartite_gnm_c_n5_n5_m25_all_complete",
        "origin": "tests/unit/igraph_bipartite_game.c — gnm m=max "
        "boundary; undirected mode=all yields complete K_{5,5}",
        "algo": "bipartite_game_gnm",
        "params": {
            "n1": 5,
            "n2": 5,
            "m": 25,
            "directed": False,
            "mode": "all",
            "seed": 5_550_006,
        },
        "expected": {
            "vcount": 10,
            "n1": 5,
            "n2": 5,
            "directed": False,
            "is_simple": True,
            "ecount_min": 25,
            "ecount_max": 25,
            "bipartite_partitions": True,
        },
    },
]

# ALGO-GN-031: iea_game. Mirrors `igraph_iea_game` in
# games/erdos_renyi.c. The IEA model assigns each edge independently
# to an ordered vertex pair (uniformly over [0,n)^2, or [0,n)×[0,n)\diag
# when loops=false). Result has exactly m edges (multi-edges allowed,
# self-loops controlled by `loops`). RNG state is not portable across
# C/py/R, so we capture structural invariants only: vcount==n,
# ecount==m EXACT, directedness preserved, and (when loops=false) every
# edge must connect distinct vertices.
IEA_GAME_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "iea_c_directed_loops_n40_m100",
        "origin": "games/erdos_renyi.c — iea_game directed multigraph "
        "with self-loops allowed (n=40, m=100)",
        "algo": "iea_game",
        "params": {
            "n": 40,
            "m": 100,
            "directed": True,
            "loops": True,
            "seed": 5_551_001,
        },
        "expected": {
            "vcount": 40,
            "ecount": 100,
            "directed": True,
            "no_self_loops": False,
        },
    },
    {
        "case": "iea_c_directed_no_loops_n20_m80",
        "origin": "games/erdos_renyi.c — iea_game directed multigraph "
        "without self-loops (n=20, m=80)",
        "algo": "iea_game",
        "params": {
            "n": 20,
            "m": 80,
            "directed": True,
            "loops": False,
            "seed": 5_551_002,
        },
        "expected": {
            "vcount": 20,
            "ecount": 80,
            "directed": True,
            "no_self_loops": True,
        },
    },
    {
        "case": "iea_c_undirected_no_loops_n15_m30",
        "origin": "games/erdos_renyi.c — iea_game undirected multigraph "
        "without self-loops (n=15, m=30)",
        "algo": "iea_game",
        "params": {
            "n": 15,
            "m": 30,
            "directed": False,
            "loops": False,
            "seed": 5_551_003,
        },
        "expected": {
            "vcount": 15,
            "ecount": 30,
            "directed": False,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-014: preference_game. Mirrors `igraph_preference_game` in
# games/preference.c (Faust–Wasserman block model). RNG state is not
# portable across implementations, so we capture structural invariants
# only — vcount, directed flag, types-in-range, no_loops/no_multiple
# (when expected.is_simple), per-block edge containment when the pref
# matrix is block-diagonal, and a generous ecount band.
PREFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "preference_c_undirected_n1000_3types_diag02",
        "origin": "tests/unit/igraph_preference_game.c:46-65 — "
        "n=1000, types=3, type_dist=(1,1,1), pref diag=0.2, "
        "undirected, no loops",
        "algo": "preference_game",
        "params": {
            "nodes": 1000,
            "types": 3,
            "type_dist": [1.0, 1.0, 1.0],
            "fixed_sizes": False,
            "pref_matrix": [
                [0.2, 0.0, 0.0],
                [0.0, 0.2, 0.0],
                [0.0, 0.0, 0.2],
            ],
            "directed": False,
            "loops": False,
            "seed": 7_770_001,
        },
        "expected": {
            "vcount": 1000,
            "directed": False,
            "is_simple": True,
            # Each diagonal block is K(~333) with edge probability 0.2.
            # E[edges] ≈ 3 · C(333,2) · 0.2 ≈ 33222, give a wide band.
            "ecount_min": 25_000,
            "ecount_max": 42_000,
            "diagonal_only_pref": True,
            "max_type": 2,
        },
    },
    {
        "case": "preference_c_undirected_loops_p1_n100",
        "origin": "tests/unit/igraph_preference_game.c:97-114 — "
        "n=100, types=3, pref diag=1.0, undirected with loops; "
        "ecount lower bound 1395 from upstream assertion",
        "algo": "preference_game",
        "params": {
            "nodes": 100,
            "types": 3,
            "type_dist": [1.0, 1.0, 1.0],
            "fixed_sizes": False,
            "pref_matrix": [
                [1.0, 0.1, 0.1],
                [0.1, 1.0, 0.1],
                [0.1, 0.1, 1.0],
            ],
            "directed": False,
            "loops": True,
            "seed": 7_770_002,
        },
        "expected": {
            "vcount": 100,
            "directed": False,
            "is_simple": False,
            # 3 diag blocks × C(~33+loops,2) at p=1, plus off-diag at 0.1.
            "ecount_min": 1_395,
            "ecount_max": 5_500,
            "diagonal_only_pref": False,
            "max_type": 2,
        },
    },
    {
        "case": "preference_c_fixed_sizes_n50_9types_pathlike",
        "origin": "tests/unit/igraph_preference_game.c:139-160 — "
        "n=50 split evenly into 9 types, off-tridiagonal pref 0.1, "
        "undirected, no loops",
        "algo": "preference_game",
        "params": {
            "nodes": 50,
            "types": 9,
            "type_dist": None,
            "fixed_sizes": True,
            "pref_matrix": [
                [0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.1, 0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 0.1, 0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.1, 0.0, 0.1, 0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.1, 0.0, 0.1, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0, 0.1, 0.0, 0.1, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.0, 0.1, 0.0],
                [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.0, 0.1],
                [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.0],
            ],
            "directed": False,
            "loops": False,
            "seed": 7_770_003,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            "is_simple": True,
            # 8 connecting type-pairs, each ~ 5*6=30 or 6*6=36 slots,
            # at p=0.1 ≈ 24-30 edges total; allow wide band for RNG.
            "ecount_min": 8,
            "ecount_max": 80,
            "diagonal_only_pref": False,
            "max_type": 8,
        },
    },
]

ESTABLISHMENT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "establishment_c_zero_pref_no_edges_n20_2types",
        "origin": "tests/unit/igraph_establishment_game.c:58-62 — "
        "n=20, types=2, k=5, type_dist=(1,0), pref diag (0,1), "
        "undirected; only type 0 sampled, p_00=0 so no edges",
        "algo": "establishment_game",
        "params": {
            "nodes": 20,
            "types": 2,
            "k": 5,
            "type_dist": [1.0, 0.0],
            "pref_matrix": [
                [0.0, 0.0],
                [0.0, 1.0],
            ],
            "directed": False,
            "seed": 9_990_001,
        },
        "expected": {
            "vcount": 20,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
            "max_type": 0,
        },
    },
    {
        "case": "establishment_c_bipartite_directed_n20_2types_cross_pref",
        "origin": "tests/unit/igraph_establishment_game.c:65-72 — "
        "n=20, types=2, k=5, type_dist=(1,1), pref off-diag 1, "
        "directed; produces a bipartite directed graph",
        "algo": "establishment_game",
        "params": {
            "nodes": 20,
            "types": 2,
            "k": 5,
            "type_dist": [1.0, 1.0],
            "pref_matrix": [
                [0.0, 1.0],
                [1.0, 0.0],
            ],
            "directed": True,
            "seed": 9_990_002,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "is_simple": True,
            # Vertices [k, n) each contribute 0..k cross edges; band wide.
            "ecount_min": 0,
            "ecount_max": 75,  # (n-k)*k = 75 upper bound
            "cross_only_pref": True,
            "max_type": 1,
        },
    },
    {
        "case": "establishment_c_full_p1_n50_3types_k4",
        "origin": "constructed (mirrors igraph_establishment_game(n=50, "
        "types=3, k=4, type_dist=(1,1,1), pref_matrix=ones)): every "
        "candidate edge accepts ⇒ exactly (n-k)*k edges",
        "algo": "establishment_game",
        "params": {
            "nodes": 50,
            "types": 3,
            "k": 4,
            "type_dist": [1.0, 1.0, 1.0],
            "pref_matrix": [
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            "directed": False,
            "seed": 9_990_003,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            "is_simple": True,
            "ecount_min": 184,  # (50-4)*4 = 184
            "ecount_max": 184,
            "max_type": 2,
        },
    },
]

CALLAWAY_TRAITS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "callaway_c_zero_pref_no_edges_n30_2types",
        "origin": "constructed (mirrors igraph_callaway_traits_game(n=30, "
        "types=2, edges_per_step=4, type_dist=(1,1), pref=zeros, "
        "undirected)) — every candidate edge rejected",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 30,
            "types": 2,
            "edges_per_step": 4,
            "type_dist": [1.0, 1.0],
            "pref_matrix": [
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            "directed": False,
            "seed": 9_991_001,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "ecount_min": 0,
            "ecount_max": 0,
            "max_type": 1,
        },
    },
    {
        "case": "callaway_c_full_p1_n40_3types_eps3",
        "origin": "constructed (mirrors igraph_callaway_traits_game(n=40, "
        "types=3, edges_per_step=3, type_dist=(1,1,1), pref=ones, "
        "undirected)) — every candidate accepted ⇒ exactly (n-1)*eps edges",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 40,
            "types": 3,
            "edges_per_step": 3,
            "type_dist": [1.0, 1.0, 1.0],
            "pref_matrix": [
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            "directed": False,
            "seed": 9_991_002,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "ecount_min": 117,  # (40-1)*3 = 117
            "ecount_max": 117,
            "max_type": 2,
        },
    },
    {
        "case": "callaway_c_diag_only_directed_n50_3types_eps2",
        "origin": "constructed (mirrors igraph_callaway_traits_game(n=50, "
        "types=3, edges_per_step=2, type_dist=(1,1,1), pref diag-only at 1, "
        "directed)) — accepted edges connect same-type vertices only",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 50,
            "types": 3,
            "edges_per_step": 2,
            "type_dist": [1.0, 1.0, 1.0],
            "pref_matrix": [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            "directed": True,
            "seed": 9_991_003,
        },
        "expected": {
            "vcount": 50,
            "directed": True,
            "ecount_min": 0,
            "ecount_max": 98,  # (50-1)*2 = 98 upper bound
            "diagonal_only_pref": True,
            "max_type": 2,
        },
    },
]

CITED_TYPE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "cited_type_c_uniform_pref_n20_2types_eps2_directed",
        "origin": "constructed (mirrors igraph_cited_type_game(n=20, "
        "types=[0,1,0,1,...], pref=[1,1], edges_per_step=2, directed)) — "
        "uniform pref ⇒ exactly (n-1)*eps edges, no self-loops",
        "algo": "cited_type_game",
        "params": {
            "nodes": 20,
            "types": [v % 2 for v in range(20)],
            "pref": [1.0, 1.0],
            "edges_per_step": 2,
            "directed": True,
            "seed": 9_992_001,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "ecount_min": 38,  # (20-1)*2 = 38
            "ecount_max": 38,
            "no_self_loops": True,
            "max_type": 1,
        },
    },
    {
        "case": "cited_type_c_skewed_pref_n30_3types_eps3_undirected",
        "origin": "constructed (mirrors igraph_cited_type_game(n=30, "
        "types=[0,1,2,...], pref=[3.0,1.0,0.1], edges_per_step=3, undirected)) — "
        "positive pref everywhere ⇒ (n-1)*eps edges, no self-loops",
        "algo": "cited_type_game",
        "params": {
            "nodes": 30,
            "types": [v % 3 for v in range(30)],
            "pref": [3.0, 1.0, 0.1],
            "edges_per_step": 3,
            "directed": False,
            "seed": 9_992_002,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "ecount_min": 87,  # (30-1)*3 = 87
            "ecount_max": 87,
            "no_self_loops": True,
            "max_type": 2,
        },
    },
    {
        "case": "cited_type_c_zero_pref_self_loop_fallback_n10",
        "origin": "constructed (mirrors igraph_cited_type_game(n=10, "
        "types=[0]*10, pref=[0.0], edges_per_step=2, undirected)) — "
        "sum=0 fallback ⇒ every citation is a self-loop on the step vertex",
        "algo": "cited_type_game",
        "params": {
            "nodes": 10,
            "types": [0 for _ in range(10)],
            "pref": [0.0],
            "edges_per_step": 2,
            "directed": False,
            "seed": 9_992_003,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount_min": 18,  # (10-1)*2 = 18
            "ecount_max": 18,
            "all_self_loops": True,
            "max_type": 0,
        },
    },
]

CITING_CITED_TYPE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "citing_cited_c_identity_pref_n20_2types_eps2_directed",
        "origin": "constructed (mirrors igraph_citing_cited_type_game(n=20, "
        "types=[0,1,0,1,...], pref=[[1,1],[1,1]], edges_per_step=2, directed)) — "
        "uniform 2x2 pref ⇒ exactly (n-1)*eps edges, no self-loops",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 20,
            "types": [v % 2 for v in range(20)],
            "pref": [[1.0, 1.0], [1.0, 1.0]],
            "edges_per_step": 2,
            "directed": True,
            "seed": 9_993_001,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "ecount_min": 38,  # (20-1)*2 = 38
            "ecount_max": 38,
            "no_self_loops": True,
            "max_type": 1,
        },
    },
    {
        "case": "citing_cited_c_assortative_pref_n30_3types_eps3_undirected",
        "origin": "constructed (mirrors igraph_citing_cited_type_game(n=30, "
        "types=[0,1,2,...], pref diagonal-dominant 3x3, edges_per_step=3, "
        "undirected)) — strictly positive pref ⇒ (n-1)*eps edges, no self-loops",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 30,
            "types": [v % 3 for v in range(30)],
            "pref": [
                [10.0, 0.1, 0.1],
                [0.1, 10.0, 0.1],
                [0.1, 0.1, 10.0],
            ],
            "edges_per_step": 3,
            "directed": False,
            "seed": 9_993_002,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "ecount_min": 87,  # (30-1)*3 = 87
            "ecount_max": 87,
            "no_self_loops": True,
            "max_type": 2,
        },
    },
    {
        "case": "citing_cited_c_zero_pref_uniform_fallback_n12_directed",
        "origin": "constructed (mirrors igraph_citing_cited_type_game(n=12, "
        "types=[0]*12, pref=[[0.0]], edges_per_step=2, directed)) — sum=0 ⇒ "
        "uniform fallback RNG_INTEGER(0, i-1) ⇒ every target strictly less "
        "than its source, NEVER a self-loop (contrast cited_type which "
        "self-loops in this regime)",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 12,
            "types": [0 for _ in range(12)],
            "pref": [[0.0]],
            "edges_per_step": 2,
            "directed": True,
            "seed": 9_993_003,
        },
        "expected": {
            "vcount": 12,
            "directed": True,
            "ecount_min": 22,  # (12-1)*2 = 22
            "ecount_max": 22,
            "no_self_loops": True,
            "max_type": 0,
        },
    },
]

LASTCIT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lastcit_c_uniform_pref_n25_3bins_eps2_directed",
        "origin": "constructed (mirrors igraph_lastcit_game(n=25, "
        "edges_per_node=2, agebins=3, preference=[1,1,1,1], directed)) — "
        "uniform preference (all bins equally attractive) so every step emits "
        "exactly eps edges and the psumtree.search picks a vertex uniformly "
        "weighted by current weights",
        "algo": "lastcit_game",
        "params": {
            "nodes": 25,
            "edges_per_node": 2,
            "agebins": 3,
            "preference": [1.0, 1.0, 1.0, 1.0],
            "directed": True,
            "seed": 9_993_001,
        },
        "expected": {
            "vcount": 25,
            "directed": True,
            "ecount_min": 48,  # (25-1)*2 = 48
            "ecount_max": 48,
            "no_self_loops": True,
        },
    },
    {
        "case": "lastcit_c_recency_decay_n40_4bins_eps3_undirected",
        "origin": "constructed (mirrors igraph_lastcit_game(n=40, "
        "edges_per_node=3, agebins=4, preference=[8,4,2,1,0.5], undirected)) — "
        "sharply decaying preference favours recently-cited vertices; "
        "never-cited bucket positive (0.5) keeps the psumtree non-zero throughout",
        "algo": "lastcit_game",
        "params": {
            "nodes": 40,
            "edges_per_node": 3,
            "agebins": 4,
            "preference": [8.0, 4.0, 2.0, 1.0, 0.5],
            "directed": False,
            "seed": 9_993_002,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "ecount_min": 117,  # (40-1)*3 = 117
            "ecount_max": 117,
            "no_self_loops": True,
        },
    },
    {
        "case": "lastcit_c_only_uncited_pref_n30_eps1",
        "origin": "constructed (mirrors igraph_lastcit_game(n=30, "
        "edges_per_node=1, agebins=2, preference=[0,0,1], directed)) — "
        "only never-cited bucket carries weight, so once a vertex is cited "
        "its weight drops to 0; with eps=1 the dst sequence is therefore a "
        "permutation prefix of the never-cited pool",
        "algo": "lastcit_game",
        "params": {
            "nodes": 30,
            "edges_per_node": 1,
            "agebins": 2,
            "preference": [0.0, 0.0, 1.0],
            "directed": True,
            "seed": 9_993_003,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "ecount_min": 29,  # (30-1)*1 = 29
            "ecount_max": 29,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-019: recent_degree_game. Mirrors
# igraph_recent_degree_game (references/igraph/src/games/recent_degree.c):
# sliding-window preferential attachment where each vertex's draw weight
# is `pow(recent_in_degree, power) + zero_appeal`. Edges added at step
# `i - time_window` are expired from the psum tree at step `i`. RNG state
# is not portable; fixtures pin our SplitMix64 output and assert
# structural invariants. Never self-loops by construction (psumtree
# ranges over [0, i) before vertex i is inserted).
RECENT_DEGREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "recent_degree_c_pow15_window5_m3_directed",
        "origin": "constructed (mirrors igraph_recent_degree_game(n=30, "
        "power=1.5, time_window=5, m=3, zero_appeal=1.0, directed)) — "
        "strong recency preference inside a short 5-step window; ecount = "
        "(30-1)*3 = 87",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 30,
            "power": 1.5,
            "time_window": 5,
            "m": 3,
            "outpref": False,
            "zero_appeal": 1.0,
            "directed": True,
            "seed": 9_995_001,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "ecount_min": 87,  # (30-1)*3 = 87
            "ecount_max": 87,
            "no_self_loops": True,
        },
    },
    {
        "case": "recent_degree_c_uniform_pow0_no_expiry_m2_undirected",
        "origin": "constructed (mirrors igraph_recent_degree_game(n=40, "
        "power=0.0, time_window=40, m=2, zero_appeal=1.0, undirected)) — "
        "power=0 means all weights collapse to zero_appeal, so the draw "
        "is uniform over existing vertices; window=n keeps the BIT-tree "
        "from ever expiring",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 40,
            "power": 0.0,
            "time_window": 40,
            "m": 2,
            "outpref": False,
            "zero_appeal": 1.0,
            "directed": False,
            "seed": 9_995_002,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "ecount_min": 78,  # (40-1)*2 = 78
            "ecount_max": 78,
            "no_self_loops": True,
        },
    },
    {
        "case": "recent_degree_c_outpref_pow2_window10_m4_directed",
        "origin": "constructed (mirrors igraph_recent_degree_game(n=25, "
        "power=2.0, time_window=10, m=4, outpref=true, zero_appeal=0.5, "
        "directed)) — outpref=true makes the source's outgoing citations "
        "also count toward its recent in-degree; exercises the source-weight "
        "refresh branch",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 25,
            "power": 2.0,
            "time_window": 10,
            "m": 4,
            "outpref": True,
            "zero_appeal": 0.5,
            "directed": True,
            "seed": 9_995_003,
        },
        "expected": {
            "vcount": 25,
            "directed": True,
            "ecount_min": 96,  # (25-1)*4 = 96
            "ecount_max": 96,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-020: barabasi_game_psumtree / barabasi_game_psumtree_multiple.
# Mirrors igraph_i_barabasi_game_psumtree and ..._psumtree_multiple
# (references/igraph/src/games/barabasi.c ~195-414): Fenwick-BIT-based
# preferential attachment. The SIMPLE variant zeros each chosen vertex's
# weight per draw to prevent within-step duplicates; the MULTIPLE variant
# snapshots the BIT sum once per step and may produce within-step
# multi-edges (the saturation branch fires when m >= i, emitting only `i`
# edges instead of `m`). RNG state is not portable; fixtures pin our
# SplitMix64 output and assert structural invariants. Never self-loops by
# construction (the binary-lifted prefix search is bounded to [0, i)
# before vertex i is added).
BARABASI_PSUMTREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_psumtree_c_classic_directed_m2",
        "origin": "constructed (mirrors igraph_barabasi_game(n=40, "
        "power=1.0, m=2, outpref=false, A=1.0, directed, algo=PSUMTREE)) — "
        "classical BA kernel (linear-in-degree weights) with simple "
        "variant; ecount = (40-1)*2 = 78 by construction",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 40,
            "power": 1.0,
            "m": 2,
            "outpref": False,
            "a": 1.0,
            "directed": True,
            "variant": "psumtree",
            "seed": 9_998_001,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "ecount_min": 78,  # (40-1)*2 = 78
            "ecount_max": 78,
            "no_self_loops": True,
        },
    },
    {
        "case": "barabasi_psumtree_c_multiple_pow15_directed_m3",
        "origin": "constructed (mirrors igraph_barabasi_game(n=30, "
        "power=1.5, m=3, outpref=false, A=1.0, directed, "
        "algo=PSUMTREE_MULTIPLE)) — non-linear (super-linear, alpha=1.5) "
        "attachment with multi-edge variant; the saturation branch fires "
        "for steps i in {1, 2, 3} (emitting 1+2+3=6 edges instead of 9), "
        "so ecount = 29*3 - 3*2/2 = 84",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 30,
            "power": 1.5,
            "m": 3,
            "outpref": False,
            "a": 1.0,
            "directed": True,
            "variant": "psumtree_multiple",
            "seed": 9_998_002,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "ecount_min": 84,  # (30-1)*3 - 3*(3-1)/2 = 84
            "ecount_max": 84,
            "no_self_loops": True,
        },
    },
    {
        "case": "barabasi_psumtree_c_undirected_outpref_m2",
        "origin": "constructed (mirrors igraph_barabasi_game(n=35, "
        "power=1.0, m=2, outpref=true, A=0.5, undirected, algo=PSUMTREE)) "
        "— undirected forces outpref=true (so A=0.5 is fine); simple "
        "variant; ecount = (35-1)*2 = 68",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 35,
            "power": 1.0,
            "m": 2,
            "outpref": True,
            "a": 0.5,
            "directed": False,
            "variant": "psumtree",
            "seed": 9_998_003,
        },
        "expected": {
            "vcount": 35,
            "directed": False,
            "ecount_min": 68,  # (35-1)*2 = 68
            "ecount_max": 68,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-021: barabasi_aging_game. Mirrors igraph_barabasi_aging_game
# (references/igraph/src/games/barabasi.c ~606-841): PsumTree-based BA
# with vertex aging. Weight = (deg_coef · pow(deg, pa_exp) + zero_deg_appeal)
# · (age_coef · pow(age, aging_exp) + zero_age_appeal). Age is binned with
# binwidth = nodes/aging_bins + 1, and the age sweep at every k*binwidth
# boundary refreshes vertex `i - k*binwidth`. Without outseq, ecount =
# (nodes - 1) * m exactly. Never self-loops by construction (search_bounded
# clamps to [0, i) before vertex i joins the BIT). Within-step multi-edges
# can occur when m >= 2 because the C source does NOT zero picks per draw.
BARABASI_AGING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_aging_c_classic_no_aging_directed_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=40, "
        "m=2, outpref=false, pa_exp=1.0, aging_exp=0.0, aging_bins=10, "
        "zero_deg_appeal=1.0, zero_age_appeal=1.0, deg_coef=1.0, "
        "age_coef=1.0, directed=true)) — degenerate case: aging_exp=0 "
        "collapses age term to (1·1 + 1) = 2 constant, so weights "
        "reduce to 2·(deg + 1), classical BA up to a constant factor; "
        "ecount = (40-1)*2 = 78",
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
            "seed": 9_998_101,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "ecount_min": 78,  # (40-1)*2 = 78
            "ecount_max": 78,
            "no_self_loops": True,
        },
    },
    {
        "case": "barabasi_aging_c_strong_aging_directed_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=40, "
        "m=2, outpref=false, pa_exp=1.0, aging_exp=-1.0, aging_bins=10, "
        "zero_deg_appeal=1.0, zero_age_appeal=1.0, deg_coef=1.0, "
        "age_coef=1.0, directed=true)) — aging_exp=-1 suppresses old "
        "vertices linearly with age bin; ecount = (40-1)*2 = 78 still "
        "exact by construction (one edge per attempted draw)",
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
            "seed": 9_998_102,
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
        "case": "barabasi_aging_c_outpref_undirected_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=35, "
        "m=2, outpref=true, pa_exp=1.0, aging_exp=-0.5, aging_bins=8, "
        "zero_deg_appeal=0.5, zero_age_appeal=1.0, deg_coef=1.0, "
        "age_coef=1.0, directed=false)) — undirected + outpref so the "
        "new vertex's own out-degree feeds back into its weight; ecount "
        "= (35-1)*2 = 68",
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
            "seed": 9_998_103,
        },
        "expected": {
            "vcount": 35,
            "directed": False,
            "ecount_min": 68,  # (35-1)*2 = 68
            "ecount_max": 68,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-032: recent_degree_aging_game. Mirrors
# igraph_recent_degree_aging_game (references/igraph/src/games/
# recent_degree.c lines 228-381). Hybrid of GN-019 (recent-degree FIFO
# sliding window) and GN-021 (vertex aging with binwidth-based age bins).
# Weight = (pow(recent_deg, pa_exp) + zero_appeal) * pow(age, aging_exp).
# Without outseq, ecount = (nodes - 1) * m exactly. Never self-loops by
# construction (search_bounded clamps to [0, i)). RNG is not portable, so
# conformance is structural only: vcount, ecount exact, no_self_loops.
RECENT_DEGREE_AGING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "recent_degree_aging_c_no_aging_directed_m2",
        "origin": "constructed (mirrors igraph_recent_degree_aging_game("
        "n=40, m=2, outpref=false, pa_exp=1.0, aging_exp=0.0, "
        "aging_bins=10, time_window=5, zero_appeal=1.0, directed=true)) "
        "— aging_exp=0 collapses age term to 1 constant, so weights "
        "reduce to (deg + 1), BA-like with recent-degree window; "
        "ecount = (40-1)*2 = 78",
        "algo": "recent_degree_aging_game",
        "params": {
            "nodes": 40,
            "m": 2,
            "outpref": False,
            "pa_exp": 1.0,
            "aging_exp": 0.0,
            "aging_bins": 10,
            "time_window": 5,
            "zero_appeal": 1.0,
            "directed": True,
            "seed": 9_997_101,
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
        "case": "recent_degree_aging_c_strong_aging_directed_m2",
        "origin": "constructed (mirrors igraph_recent_degree_aging_game("
        "n=40, m=2, outpref=false, pa_exp=1.0, aging_exp=-1.0, "
        "aging_bins=10, time_window=8, zero_appeal=1.0, directed=true)) "
        "— aging_exp=-1 suppresses old vertices; time_window=8 means "
        "recent degree resets after 8 steps; ecount = (40-1)*2 = 78",
        "algo": "recent_degree_aging_game",
        "params": {
            "nodes": 40,
            "m": 2,
            "outpref": False,
            "pa_exp": 1.0,
            "aging_exp": -1.0,
            "aging_bins": 10,
            "time_window": 8,
            "zero_appeal": 1.0,
            "directed": True,
            "seed": 9_997_102,
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
        "case": "recent_degree_aging_c_outpref_undirected_m2",
        "origin": "constructed (mirrors igraph_recent_degree_aging_game("
        "n=35, m=2, outpref=true, pa_exp=1.0, aging_exp=-0.5, "
        "aging_bins=8, time_window=10, zero_appeal=0.5, directed=false)) "
        "— undirected + outpref feeds the new vertex's own degree back "
        "into its weight; ecount = (35-1)*2 = 68",
        "algo": "recent_degree_aging_game",
        "params": {
            "nodes": 35,
            "m": 2,
            "outpref": True,
            "pa_exp": 1.0,
            "aging_exp": -0.5,
            "aging_bins": 8,
            "time_window": 10,
            "zero_appeal": 0.5,
            "directed": False,
            "seed": 9_997_103,
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

# ALGO-GN-022: dot_product_game. Mirrors igraph_dot_product_game
# (references/igraph/src/games/dotproduct.c:59-102). Per-pair Bernoulli
# with edge probability = dot(v_i, v_j). Three fixtures cover the three
# clamp regimes deterministically so the ecount band is *exact* under
# any RNG state — conformance is structural only because SplitMix64 is
# not portable to glibc-style RNG.
#   * all_ones_complete_n8_undirected — v_i = [1.0] → dot = 1.0 → every
#     pair Bernoulli draw fires (gen_unit < 1.0 always since gen_unit ∈
#     [0,1)) so ecount = n(n-1)/2 = 28 exactly.
#   * orthogonal_groups_n8_undirected — half [1,0] + half [0,1] → same-
#     group dot = 1 (always edge), cross-group dot = 0 (never edge),
#     ecount = 2 · C(4, 2) = 12 exactly.
#   * mixed_signs_n10_undirected — half [1.5] + half [-0.5] → same-
#     group dots = 2.25 or 0.25 (always edge for +, Bernoulli 0.25 for
#     −); cross-group dots = -0.75 (skip + warn). Worst case still
#     bounds ecount ∈ [C(5,2), C(5,2) + C(5,2)] = [10, 20].
DOT_PRODUCT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dot_product_c_all_ones_complete_n8_undirected",
        "origin": "constructed (mirrors igraph_dot_product_game with "
        "vecs[i] = [1.0] for i ∈ [0, 8), directed=false) — every dot "
        "product is 1.0; under strict `gen_unit() < prob` Bernoulli "
        "(gen_unit ∈ [0, 1)) every pair fires; ecount = 8·7/2 = 28 "
        "exact",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.0]] * 8,
            "directed": False,
            "seed": 10_001_101,
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
        "case": "dot_product_c_orthogonal_groups_n8_undirected",
        "origin": "constructed (mirrors igraph_dot_product_game with "
        "vecs = [[1,0]]*4 ++ [[0,1]]*4, directed=false) — same-group "
        "dot = 1 (always edge), cross-group dot = 0 (never edge); "
        "ecount = 2·C(4,2) = 12 exact",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.0, 0.0]] * 4 + [[0.0, 1.0]] * 4,
            "directed": False,
            "seed": 10_001_102,
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
        "case": "dot_product_c_mixed_clamp_n10_directed",
        "origin": "constructed (mirrors igraph_dot_product_game with "
        "vecs = [[1.5]]*5 ++ [[-0.5]]*5, directed=true) — same-(+) dot "
        "= 2.25 always edge (no RNG draw) → 5·4 = 20; same-(−) dot = "
        "0.25 Bernoulli → 0..20; cross dot = -0.75 skip → 0; ecount ∈ "
        "[20, 40]. Exercises both warning regimes",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.5]] * 5 + [[-0.5]] * 5,
            "directed": True,
            "seed": 10_001_103,
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

# ALGO-GN-023: correlated_game + correlated_pair_game. Mirrors
# `igraph_correlated_game()` and `igraph_correlated_pair_game()` from
# references/igraph/src/games/correlated.c. The model preserves the
# marginal edge probability `p` of the input graph while injecting
# Pearson adjacency correlation `corr ∈ [0, 1]` via the 2x2 contingency
# table {p_del = 1 - q, p_add = (1 - q) * p / (1 - p), q = p + corr*(1-p)}.
# Because SplitMix64 is not portable to glibc-style RNG, fixtures are
# structural-only: corr = 1 cases pin exact ecount (copy of old graph),
# and `correlated_pair_game` fixtures use ±6σ Binomial bands.
CORRELATED_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "correlated_c_corr1_path_n4_exact_copy",
        "origin": "constructed (mirrors igraph_correlated_game with "
        "old = path P4 on n=4 vertices, corr=1.0, p=0.5, no "
        "permutation) — corr=1 yields p_del=0, p_add=0, so the new "
        "graph is an *exact copy* of the old graph; ecount = 3 exact",
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
            "seed": 11_002_301,
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
        "case": "correlated_c_corr1_cycle_n5_permutation_reverse",
        "origin": "constructed (mirrors igraph_correlated_game with "
        "old = cycle C5 on n=5 vertices, corr=1.0, p=0.5, "
        "permutation = (4,3,2,1,0)) — corr=1 keeps every edge; "
        "permutation only relabels vertices ⇒ ecount = 5 exact",
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
            "seed": 11_002_302,
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

# ALGO-GN-023: correlated_pair_game — convenience wrapper that draws an
# ER(n, p) graph and a correlated counterpart from a single seed.
# Structural-only: vcount on both graphs, simple-by-construction, and
# 6σ Binomial bands on both ecounts.
CORRELATED_PAIR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "correlated_pair_c_n30_corr5_p2_undirected",
        "origin": "constructed (mirrors igraph_correlated_pair_game with "
        "n=30, corr=0.5, p=0.2, directed=false) — both graphs ER-marginal "
        "with mean ecount = C(30,2)·0.2 = 87, σ ≈ sqrt(435·0.2·0.8) ≈ "
        "8.34, ±6σ ≈ ±50, conservative band [40, 140]",
        "algo": "correlated_pair_game",
        "params": {
            "n": 30,
            "corr": 0.5,
            "p": 0.2,
            "directed": False,
            "permutation": None,
            "seed": 11_002_311,
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
        "case": "correlated_pair_c_n20_corr8_p25_directed",
        "origin": "constructed (mirrors igraph_correlated_pair_game with "
        "n=20, corr=0.8, p=0.25, directed=true) — both graphs ER-marginal "
        "with mean ecount = 20·19·0.25 = 95, σ ≈ sqrt(380·0.25·0.75) ≈ "
        "8.44, ±6σ ≈ ±51, conservative band [45, 150]",
        "algo": "correlated_pair_game",
        "params": {
            "n": 20,
            "corr": 0.8,
            "p": 0.25,
            "directed": True,
            "permutation": None,
            "seed": 11_002_312,
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

# Configuration-model degree-sequence generator (ALGO-GN-024).
# Fixtures mirror references/igraph/tests/unit/igraph_degree_sequence_game.c
# (CONFIGURATION branch only). The C test asserts that the observed degree
# sequence matches the input *exactly* (the algorithm is degree-preserving
# by construction). Fixtures therefore pin the expected vcount, ecount and
# the full degree vector — bands not needed.
DEGREE_SEQUENCE_CONFIG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_config_c_undirected_n10_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:43-74 — "
        "outarr=[2,3,2,3,3,3,3,1,4,4] CONFIGURATION undirected; "
        "the C test asserts observed degrees match input exactly",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": None,
            "seed": 333,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 14,
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": None,
        },
    },
    {
        "case": "degseq_config_c_directed_n10_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:43-49 — "
        "outarr=[2,3,2,3,3,3,3,1,4,4] inarr=[3,6,2,0,2,2,4,3,3,3] "
        "CONFIGURATION directed; observed degrees must match input exactly",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": [3, 6, 2, 0, 2, 2, 4, 3, 3, 3],
            "seed": 333,
        },
        "expected": {
            "vcount": 10,
            "directed": True,
            "ecount": 28,
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": [3, 6, 2, 0, 2, 2, 4, 3, 3, 3],
        },
    },
    {
        "case": "degseq_config_c_empty_sequence",
        "origin": "tests/unit/igraph_degree_sequence_game.c:76-78 — "
        "empty out_degrees CONFIGURATION undirected; vcount must be 0",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [],
            "in_degrees": None,
            "seed": 333,
        },
        "expected": {
            "vcount": 0,
            "directed": False,
            "ecount": 0,
            "out_degrees": [],
            "in_degrees": None,
        },
    },
]

# Viger-Latapy degree-sequence generator (ALGO-GN-025). Fixtures mirror
# the VL block of references/igraph/tests/unit/igraph_degree_sequence_game.c
# (lines 230-256). The VL method samples a *connected, simple* undirected
# graph realising the input degree sequence; the C test asserts vcount,
# is_simple, is_connected (weak), and exact degree match. Fixtures pin
# those invariants exactly (no bands needed — they're guaranteed by the
# algorithm definition).
DEGREE_SEQUENCE_VL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_vl_c_undirected_n10_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:230-246 — "
        "outarr=[2,3,2,3,3,3,3,1,4,4] DEGSEQ_VL undirected; the C test "
        "asserts simple, connected, and exact degree match",
        "algo": "degree_sequence_game_vl",
        "params": {
            "degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "seed": 333,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 14,
            "degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "is_simple": True,
            "is_connected": True,
        },
    },
    {
        "case": "degseq_vl_c_empty_sequence",
        "origin": "tests/unit/igraph_degree_sequence_game.c:248-251 — "
        "empty degrees DEGSEQ_VL undirected; vcount must be 0",
        "algo": "degree_sequence_game_vl",
        "params": {
            "degrees": [],
            "seed": 333,
        },
        "expected": {
            "vcount": 0,
            "directed": False,
            "ecount": 0,
            "degrees": [],
            "is_simple": True,
            "is_connected": True,
        },
    },
]

# Fast-heuristic-simple degree-sequence generator (ALGO-GN-026). Fixtures
# mirror the FAST_HEUR block of
# references/igraph/tests/unit/igraph_degree_sequence_game.c (lines 160-198).
# This method samples a *simple* (no self-loops, no multi-edges) graph that
# realises the input degree sequence exactly. The C test asserts vcount,
# directedness, is_simple, and exact (out/in-)degree match. RNG state is
# not portable; fixtures pin those structural invariants only.
DEGREE_SEQUENCE_FAST_HEUR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_fastheur_c_undirected_n10_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:160-171 — "
        "outarr=[2,3,2,3,3,3,3,1,4,4] DEGSEQ_FAST_HEUR_SIMPLE undirected; "
        "the C test asserts is_simple and exact degree match",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": None,
            "seed": 333,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 14,
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_fastheur_c_undirected_empty",
        "origin": "tests/unit/igraph_degree_sequence_game.c:173-175 — "
        "empty out_degrees DEGSEQ_FAST_HEUR_SIMPLE undirected; vcount must be 0",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [],
            "in_degrees": None,
            "seed": 333,
        },
        "expected": {
            "vcount": 0,
            "directed": False,
            "ecount": 0,
            "out_degrees": [],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_fastheur_c_directed_n10_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:180-194 — "
        "outarr=[2,3,2,3,3,3,3,1,4,4] inarr=[3,6,2,0,2,2,4,3,3,3] "
        "DEGSEQ_FAST_HEUR_SIMPLE directed; is_simple and exact out/in match",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": [3, 6, 2, 0, 2, 2, 4, 3, 3, 3],
            "seed": 333,
        },
        "expected": {
            "vcount": 10,
            "directed": True,
            "ecount": 28,
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": [3, 6, 2, 0, 2, 2, 4, 3, 3, 3],
            "is_simple": True,
        },
    },
    {
        "case": "degseq_fastheur_c_directed_empty",
        "origin": "tests/unit/igraph_degree_sequence_game.c:196-198 — "
        "empty out/in DEGSEQ_FAST_HEUR_SIMPLE directed; vcount must be 0",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [],
            "in_degrees": [],
            "seed": 333,
        },
        "expected": {
            "vcount": 0,
            "directed": True,
            "ecount": 0,
            "out_degrees": [],
            "in_degrees": [],
            "is_simple": True,
        },
    },
]

# Configuration-simple degree-sequence generator (ALGO-GN-027). Fixtures
# mirror the CONFIGURATION_SIMPLE blocks of
# references/igraph/tests/unit/igraph_degree_sequence_game.c (lines 101-155).
# This method samples uniformly from simple realisations of the input degree
# sequence via the configuration model with rejection sampling on collision.
# Like FAST_HEUR_SIMPLE the C test asserts vcount, directedness, is_simple,
# and exact (out/in-)degree match — RNG state is not portable, so we pin
# only those structural invariants.
DEGREE_SEQUENCE_CONFIG_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_configsimple_c_undirected_n10_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:103-114 — "
        "outarr=[2,3,2,3,3,3,3,1,4,4] DEGSEQ_CONFIGURATION_SIMPLE undirected; "
        "C test asserts !directed, vcount==n, is_simple, exact degree match",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": None,
            "seed": 333,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 14,
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_configsimple_c_undirected_empty",
        "origin": "tests/unit/igraph_degree_sequence_game.c:116-118 — "
        "empty out_degrees DEGSEQ_CONFIGURATION_SIMPLE undirected; vcount must be 0",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [],
            "in_degrees": None,
            "seed": 333,
        },
        "expected": {
            "vcount": 0,
            "directed": False,
            "ecount": 0,
            "out_degrees": [],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_configsimple_c_directed_n8_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:137-151 — "
        "directed DEGSEQ_CONFIGURATION_SIMPLE invariants (is_simple, exact "
        "out/in match). The C reference uses outarr=[2,3,2,3,3,3,3,1,4,4] "
        "inarr=[3,6,2,0,2,2,4,3,3,3] (Σ=28 on n=10), which yields "
        "exp(~7.8) ≈ 2440 expected restarts and exceeds our "
        "MAX_OUTER_ATTEMPTS=1024 budget; this fixture preserves the same "
        "structural assertions on a lower-density directed sequence "
        "(out=[2,2,2,1,1,1,1,1] in=[1,2,1,1,2,1,2,1], Σ=10 on n=8) so the "
        "rejection sampler can run reliably.",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [2, 2, 2, 1, 1, 1, 1, 1],
            "in_degrees": [1, 2, 1, 1, 2, 1, 2, 1],
            "seed": 333,
        },
        "expected": {
            "vcount": 8,
            "directed": True,
            "ecount": 11,
            "out_degrees": [2, 2, 2, 1, 1, 1, 1, 1],
            "in_degrees": [1, 2, 1, 1, 2, 1, 2, 1],
            "is_simple": True,
        },
    },
    {
        "case": "degseq_configsimple_c_directed_empty",
        "origin": "tests/unit/igraph_degree_sequence_game.c:153-155 — "
        "empty out/in DEGSEQ_CONFIGURATION_SIMPLE directed; vcount must be 0",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [],
            "in_degrees": [],
            "seed": 333,
        },
        "expected": {
            "vcount": 0,
            "directed": True,
            "ecount": 0,
            "out_degrees": [],
            "in_degrees": [],
            "is_simple": True,
        },
    },
]

# Edge-switching MCMC simple-graph degree-sequence generator (ALGO-GN-028).
# Fixtures mirror the EDGE_SWITCHING_SIMPLE blocks of
# references/igraph/tests/unit/igraph_degree_sequence_game.c (lines 200-227).
# Two-phase algorithm: deterministic Havel-Hakimi INDEX (undirected) or
# Kleitman-Wang INDEX (directed) seed, followed by 10·|E| degree-preserving
# edge-switching MCMC trials. Unlike CONFIGURATION_SIMPLE (ALGO-GN-027) the
# cost is linear in |E| regardless of density, so dense / skewed sequences
# that exceed CONFIGURATION_SIMPLE's restart budget run reliably here. RNG
# state is not portable, so we pin only structural invariants: vcount,
# directedness, ecount, exact (out/in-)degree match, is_simple.
DEGREE_SEQUENCE_EDGE_SWITCHING_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_edge_switching_c_undirected_n10_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:201-211 — "
        "outarr=[2,3,2,3,3,3,3,1,4,4] DEGSEQ_EDGE_SWITCHING_SIMPLE "
        "undirected; C test asserts is_simple, exact degree match. "
        "Edge-switching handles this density (Σd/n=2.8) without "
        "restart trouble (unlike CONFIGURATION_SIMPLE).",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": None,
            "seed": 333,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 14,
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_edge_switching_c_undirected_empty",
        "origin": "constructed — empty out_degrees "
        "DEGSEQ_EDGE_SWITCHING_SIMPLE undirected; vcount must be 0 "
        "(early-exit branch in both upstream and Rust).",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [],
            "in_degrees": None,
            "seed": 333,
        },
        "expected": {
            "vcount": 0,
            "directed": False,
            "ecount": 0,
            "out_degrees": [],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_edge_switching_c_directed_n10_mixed",
        "origin": "tests/unit/igraph_degree_sequence_game.c:213-227 — "
        "directed DEGSEQ_EDGE_SWITCHING_SIMPLE invariants (is_simple, "
        "exact out/in match). Uses the upstream outarr=[2,3,2,3,3,3,3,1,"
        "4,4] / inarr=[3,6,2,0,2,2,4,3,3,3] verbatim (Σ=28, n=10) — "
        "EDGE_SWITCHING_SIMPLE handles this density linearly in |E|, "
        "no restart-cliff like CONFIGURATION_SIMPLE.",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": [3, 6, 2, 0, 2, 2, 4, 3, 3, 3],
            "seed": 333,
        },
        "expected": {
            "vcount": 10,
            "directed": True,
            "ecount": 28,
            "out_degrees": [2, 3, 2, 3, 3, 3, 3, 1, 4, 4],
            "in_degrees": [3, 6, 2, 0, 2, 2, 4, 3, 3, 3],
            "is_simple": True,
        },
    },
    {
        "case": "degseq_edge_switching_c_directed_empty",
        "origin": "constructed — empty out/in "
        "DEGSEQ_EDGE_SWITCHING_SIMPLE directed; vcount must be 0 "
        "(early-exit branch).",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [],
            "in_degrees": [],
            "seed": 333,
        },
        "expected": {
            "vcount": 0,
            "directed": True,
            "ecount": 0,
            "out_degrees": [],
            "in_degrees": [],
            "is_simple": True,
        },
    },
]

ASYMMETRIC_PREFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "asym_preference_c_full_p1_no_loops_n100_2x3",
        "origin": "tests/unit/igraph_preference_game.c:170-191 — "
        "n=100, 2x3 pref all 1, no loops; ecount = 9900 exactly",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 100,
            "no_out_types": 2,
            "no_in_types": 3,
            "type_dist_matrix": None,
            "pref_matrix": [
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            "loops": False,
            "seed": 8_880_001,
        },
        "expected": {
            "vcount": 100,
            "directed": True,
            "is_simple": True,
            # Loops removed: 100*100 - 100 = 9900.
            "ecount_min": 9_900,
            "ecount_max": 9_900,
            "max_out_type": 1,
            "max_in_type": 2,
        },
    },
    {
        "case": "asym_preference_c_full_p1_loops_n100_2x2",
        "origin": "tests/unit/igraph_preference_game.c:193-212 — "
        "n=100, 2x2 pref all 1, loops on; ecount = 10000 exactly, "
        "100 self-loops",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 100,
            "no_out_types": 2,
            "no_in_types": 2,
            "type_dist_matrix": None,
            "pref_matrix": [
                [1.0, 1.0],
                [1.0, 1.0],
            ],
            "loops": True,
            "seed": 8_880_002,
        },
        "expected": {
            "vcount": 100,
            "directed": True,
            "is_simple": False,
            "ecount_min": 10_000,
            "ecount_max": 10_000,
            "max_out_type": 1,
            "max_in_type": 1,
        },
    },
    {
        "case": "asym_preference_c_pinned_types_3x2",
        "origin": "tests/unit/igraph_preference_game.c:216-238 — "
        "n=10, type_dist_matrix pins (out=2, in=0) for every vertex",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 10,
            "no_out_types": 3,
            "no_in_types": 2,
            "type_dist_matrix": [
                [0.0, 0.0],
                [0.0, 0.0],
                [1.0, 0.0],
            ],
            "pref_matrix": [
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            "loops": True,
            "seed": 8_880_003,
        },
        "expected": {
            "vcount": 10,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
            "max_out_type": 2,
            "max_in_type": 0,
        },
    },
]

ISLANDS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "islands_c_4islands_size20_pin03_inter2",
        "origin": "constructed (mirrors igraph_simple_interconnected_islands_game"
        "(islands_n=4, islands_size=20, islands_pin=0.3, n_inter=2)): "
        "four moderate-density islands wired together",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 4,
            "islands_size": 20,
            "islands_pin": 0.3,
            "n_inter": 2,
            "seed": 5_550_001,
        },
        "expected": {
            # E[intra] = 4 · C(20,2) · 0.3 = 4 · 190 · 0.3 = 228
            # exact_inter = C(4,2) · 2 = 12
            # total ≈ 240; allow ±50% on the random part.
            "vcount": 80,
            "directed": False,
            "is_simple": True,
            "ecount_min": 124,  # 0.5 · 228 + 12 ≈ 126
            "ecount_max": 364,  # 1.5 · 228 + 12 ≈ 354 — round up
        },
    },
    {
        "case": "islands_c_pin0_only_inter",
        "origin": "constructed (mirrors igraph_simple_interconnected_islands_game"
        "(islands_pin=0)): exact inter-island count",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 5,
            "islands_size": 6,
            "islands_pin": 0.0,
            "n_inter": 3,
            "seed": 5_550_002,
        },
        "expected": {
            # No intra edges (p=0); exactly C(5,2) · 3 = 30 inter edges.
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 30,
            "ecount_max": 30,
        },
    },
    {
        "case": "islands_c_single_island_pin_one_clique",
        "origin": "constructed (mirrors igraph_simple_interconnected_islands_game"
        "(islands_n=1, islands_pin=1.0)): single clique K_15",
        "algo": "simple_interconnected_islands_game",
        "params": {
            "islands_n": 1,
            "islands_size": 15,
            "islands_pin": 1.0,
            "n_inter": 0,
            "seed": 5_550_003,
        },
        "expected": {
            "vcount": 15,
            "directed": False,
            "is_simple": True,
            "ecount_min": 105,  # K_15 = 15·14/2 = 105
            "ecount_max": 105,
        },
    },
]

K_REGULAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "k_regular_c_undirected_simple_n10_k4",
        "origin": "constructed (mirrors igraph_k_regular_game(n=10, k=4, "
        "directed=false, multiple=false)): every vertex has degree 4, "
        "graph is simple",
        "algo": "k_regular_game",
        "params": {
            "n": 10,
            "k": 4,
            "directed": False,
            "multiple": False,
            "seed": 8_880_001,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 20,  # n * k / 2
            "ecount_max": 20,
            "every_degree": 4,
        },
    },
    {
        "case": "k_regular_c_directed_simple_n8_k3",
        "origin": "constructed (mirrors igraph_k_regular_game(n=8, k=3, "
        "directed=true, multiple=false)): every vertex has out-degree "
        "= in-degree = 3, graph is simple directed",
        "algo": "k_regular_game",
        "params": {
            "n": 8,
            "k": 3,
            "directed": True,
            "multiple": False,
            "seed": 8_880_002,
        },
        "expected": {
            "vcount": 8,
            "directed": True,
            "is_simple": True,
            "ecount_min": 24,  # n * k
            "ecount_max": 24,
            "every_out_degree": 3,
            "every_in_degree": 3,
        },
    },
    {
        "case": "k_regular_c_undirected_multi_n5_k6",
        "origin": "constructed (mirrors igraph_k_regular_game(n=5, k=6, "
        "directed=false, multiple=true)): every vertex has degree 6 in "
        "a multigraph where self-loops and parallel edges are allowed",
        "algo": "k_regular_game",
        "params": {
            "n": 5,
            "k": 6,
            "directed": False,
            "multiple": True,
            "seed": 8_880_003,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "is_simple": False,  # multigraph is allowed to have loops / parallels
            "ecount_min": 15,  # n * k / 2 = 30 / 2
            "ecount_max": 15,
            "every_degree": 6,
        },
    },
]

WATTS_STROGATZ_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "watts_c_ring_lattice_p0_size20_nei2",
        "origin": "constructed (mirrors igraph_watts_strogatz_game(dim=1, "
        "size=20, nei=2, p=0)): pure 1-D ring lattice, every vertex has "
        "degree 4, edge count = size * nei = 40",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 20,
            "nei": 2,
            "p": 0.0,
            "loops": False,
            "multiple": False,
            "seed": 9_000_001,
        },
        "expected": {
            "vcount": 20,
            "directed": False,
            "is_simple": True,
            "ecount_min": 40,  # size * nei
            "ecount_max": 40,
            "every_degree": 4,
        },
    },
    {
        "case": "watts_c_small_world_p_half_size30_nei3",
        "origin": "constructed (mirrors igraph_watts_strogatz_game(dim=1, "
        "size=30, nei=3, p=0.5, loops=false, multiple=false)): rewires "
        "half the endpoints — edge count preserved, still simple",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 30,
            "nei": 3,
            "p": 0.5,
            "loops": False,
            "multiple": False,
            "seed": 9_000_002,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 90,  # size * nei
            "ecount_max": 90,
        },
    },
    {
        "case": "watts_c_full_rewire_p1_size16_nei4",
        "origin": "constructed (mirrors igraph_watts_strogatz_game(dim=1, "
        "size=16, nei=4, p=1.0, loops=false, multiple=false)): every "
        "endpoint rewired — result is essentially a random regular-ish "
        "graph, edge count preserved",
        "algo": "watts_strogatz_game",
        "params": {
            "size": 16,
            "nei": 4,
            "p": 1.0,
            "loops": False,
            "multiple": False,
            "seed": 9_000_003,
        },
        "expected": {
            "vcount": 16,
            "directed": False,
            "is_simple": True,
            "ecount_min": 64,  # size * nei
            "ecount_max": 64,
        },
    },
]

# ALGO-GN-010: sbm_game. Mirrors `igraph_sbm_game`. RNG state is not
# portable across implementations, so each fixture pins parameter
# values and bands the structural invariants the upstream C example
# (`examples/simple/igraph_sbm_game.c`) asserts:
#   * vcount = sum(block_sizes) (exact);
#   * directed matches the flag;
#   * ecount lies in a generous band around the model mean;
#   * is_simple when loops=false and multiple=false;
#   * when the pref matrix is block-diagonal, every edge stays
#     on-diagonal (encoded via `diagonal_only_pref: true`).
SBM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "sbm_c_single_block_n20_p_three_tenths",
        "origin": "constructed (mirrors igraph_sbm_game with k=1, n=20, "
        "pref=[[0.3]], loops=false, multiple=false): single-block "
        "reduces to undirected G(n,p)",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [[0.3]],
            "block_sizes": [20],
            "directed": False,
            "loops": False,
            "multiple": False,
            "seed": 10_000_001,
        },
        "expected": {
            "vcount": 20,
            "directed": False,
            "is_simple": True,
            "ecount_min": 25,
            "ecount_max": 110,
        },
    },
    {
        "case": "sbm_c_two_blocks_balanced_assortative",
        "origin": "constructed (mirrors igraph_sbm_game with k=2, "
        "sizes=[15, 15], in-block p=0.4, between-block p=0.05, "
        "undirected, no loops, no multiple): canonical assortative "
        "two-community SBM",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [[0.4, 0.05], [0.05, 0.4]],
            "block_sizes": [15, 15],
            "directed": False,
            "loops": False,
            "multiple": False,
            "seed": 10_000_002,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 60,
            "ecount_max": 150,
        },
    },
    {
        "case": "sbm_c_block_diagonal_pref_three_blocks",
        "origin": "constructed (mirrors igraph_sbm_game with k=3, "
        "sizes=[10, 10, 10], block-diagonal pref (in-block 0.3, "
        "off-diagonal 0.0)): every realised edge stays inside a "
        "block — checks block_of(u) == block_of(v) invariant",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [[0.3, 0.0, 0.0], [0.0, 0.3, 0.0], [0.0, 0.0, 0.3]],
            "block_sizes": [10, 10, 10],
            "directed": False,
            "loops": False,
            "multiple": False,
            "seed": 10_000_003,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": True,
            "ecount_min": 15,
            "ecount_max": 75,
            "diagonal_only_pref": True,
        },
    },
]

# ALGO-GN-011: hsbm_game (uniform-per-macro Hierarchical SBM). Mirrors
# `igraph_hsbm_game` in references/igraph/src/games/sbm.c and the three
# deterministic fixtures in references/igraph/tests/unit/igraph_hsbm_game.out.
# Each macro-block has the same micro-block structure (m, rho, C). The
# Rust port is deterministic per `seed`; the C oracle does not share RNG
# state, so we encode the structural invariants the C test asserts:
#   * vcount = n (exact);
#   * directed = false (HSBM is always undirected);
#   * for the three C deterministic fixtures the ecount is exact, so
#     band = [exact, exact];
#   * is_simple = true (HSBM produces simple graphs).
HSBM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hsbm_c_trivial_one_vertex",
        "origin": "constructed (mirrors igraph_hsbm_game with n=1, m=1, "
        "rho=[1.0], C=[[0.0]], p=0): one macro of one micro-cluster "
        "of one vertex — no edges possible",
        "algo": "hsbm_game",
        "params": {
            "n": 1,
            "m": 1,
            "rho": [1.0],
            "c": [[0.0]],
            "p": 0.0,
            "seed": 11_000_001,
        },
        "expected": {
            "vcount": 1,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "hsbm_c_one_macro_bipartite_k64",
        "origin": "constructed (mirrors igraph_hsbm_game with n=10, m=10, "
        "rho=[0.6, 0.4], C=[[0.0, 1.0], [1.0, 0.0]], p=0): single macro "
        "of two micro-clusters (size 6 and 4) with off-diagonal C=1 — "
        "exactly K_{6,4} = 24 edges",
        "algo": "hsbm_game",
        "params": {
            "n": 10,
            "m": 10,
            "rho": [0.6, 0.4],
            "c": [[0.0, 1.0], [1.0, 0.0]],
            "p": 0.0,
            "seed": 11_000_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 24,
            "ecount_max": 24,
        },
    },
    {
        "case": "hsbm_c_two_macros_bipartite_p1",
        "origin": "constructed (mirrors igraph_hsbm_game with n=10, m=5, "
        "rho=[0.6, 0.4], C=[[0.0, 1.0], [1.0, 0.0]], p=1): two macros "
        "each holding K_{3,2}=6 intra-macro edges, plus full K_{5,5}=25 "
        "inter-macro edges — exactly 6+6+25=37 edges (matches C .out)",
        "algo": "hsbm_game",
        "params": {
            "n": 10,
            "m": 5,
            "rho": [0.6, 0.4],
            "c": [[0.0, 1.0], [1.0, 0.0]],
            "p": 1.0,
            "seed": 11_000_003,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 37,
            "ecount_max": 37,
        },
    },
]

# ALGO-GN-011: hsbm_list_game (per-macro list Hierarchical SBM). Mirrors
# `igraph_hsbm_list_game` in references/igraph/src/games/sbm.c and the
# three fixtures in references/igraph/tests/unit/igraph_hsbm_list_game.out.
# Per-macro (m_i, rho_i, C_i) — generalisation of `hsbm_game` where
# every macro can have its own micro-structure.
HSBM_LIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hsbm_list_c_trivial_one_vertex",
        "origin": "constructed (mirrors igraph_hsbm_list_game with n=1, "
        "m_list=[1], rho_list=[[1.0]], c_list=[[[0.0]]], p=0): single "
        "macro of one vertex — no edges possible",
        "algo": "hsbm_list_game",
        "params": {
            "n": 1,
            "m_list": [1],
            "rho_list": [[1.0]],
            "c_list": [[[0.0]]],
            "p": 0.0,
            "seed": 11_100_001,
        },
        "expected": {
            "vcount": 1,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "hsbm_list_c_one_macro_bipartite_k64",
        "origin": "constructed (mirrors igraph_hsbm_list_game with n=10, "
        "m_list=[10], rho_list=[[0.6, 0.4]], c_list=[[[0.0, 1.0], "
        "[1.0, 0.0]]], p=0): single macro of two micro-clusters "
        "(6 and 4) bipartite — exactly K_{6,4} = 24 edges (matches C .out)",
        "algo": "hsbm_list_game",
        "params": {
            "n": 10,
            "m_list": [10],
            "rho_list": [[0.6, 0.4]],
            "c_list": [[[0.0, 1.0], [1.0, 0.0]]],
            "p": 0.0,
            "seed": 11_100_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 24,
            "ecount_max": 24,
        },
    },
    {
        "case": "hsbm_list_c_two_macros_replicated_p1",
        "origin": "constructed (mirrors igraph_hsbm_list_game with n=10, "
        "m_list=[5, 5], rho_list both = [0.6, 0.4], c_list both = "
        "[[0.0, 1.0], [1.0, 0.0]], p=1): two macros each = K_{3,2}=6, "
        "plus K_{5,5}=25 inter-macro — exactly 6+6+25=37 edges "
        "(matches C .out)",
        "algo": "hsbm_list_game",
        "params": {
            "n": 10,
            "m_list": [5, 5],
            "rho_list": [[0.6, 0.4], [0.6, 0.4]],
            "c_list": [
                [[0.0, 1.0], [1.0, 0.0]],
                [[0.0, 1.0], [1.0, 0.0]],
            ],
            "p": 1.0,
            "seed": 11_100_003,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 37,
            "ecount_max": 37,
        },
    },
]

# ALGO-GN-012: chung_lu_game (Miller-Hagberg expected-degree sampler).
# Mirrors `igraph_chung_lu_game` in references/igraph/src/games/chung_lu.c
# and the deterministic assertions in
# references/igraph/tests/unit/igraph_chung_lu_game.c. The C test pins
# vcount + directedness + (for `loops=false`) is_simple over every
# combination of {ORIGINAL, MAXENT, NR} × {undirected (in_weights=None),
# directed (in_weights=&indeg)} × {loops=false, loops=true}. RNG state is
# not portable across implementations, so the ecount band is left wide on
# the mid-density fixtures; one zero-weight fixture pins ecount=0 exactly.
CHUNG_LU_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "chung_lu_c_zero_weights_undirected_empty",
        "origin": "constructed (mirrors igraph_chung_lu_game with all-zero "
        "out_weights and in_weights=NULL): expected degree is zero "
        "everywhere → no edges can be sampled, exactly 0 edges.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [0.0, 0.0, 0.0, 0.0, 0.0],
            "in_weights": None,
            "loops": True,
            "variant": "original",
            "seed": 12_000_001,
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
        "case": "chung_lu_c_zero_weights_directed_empty",
        "origin": "constructed (mirrors igraph_chung_lu_game with all-zero "
        "out_weights and matching all-zero in_weights → directed empty graph).",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [0.0, 0.0, 0.0, 0.0, 0.0],
            "in_weights": [0.0, 0.0, 0.0, 0.0, 0.0],
            "loops": True,
            "variant": "original",
            "seed": 12_000_002,
        },
        "expected": {
            "vcount": 5,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "chung_lu_c_original_undirected_no_loops",
        "origin": "mirrors igraph_chung_lu_game(out=[1,0,2.5,2,3,2,1.5], "
        "in=NULL, loops=false, ORIGINAL): C test asserts undirected + simple. "
        "Sum(q_ij) = (Σw)²/(2Σw) = 6.0, so ecount sits in a wide model band.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 0.0, 2.5, 2.0, 3.0, 2.0, 1.5],
            "in_weights": None,
            "loops": False,
            "variant": "original",
            "seed": 12_000_003,
        },
        "expected": {
            "vcount": 7,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 21,
        },
    },
    {
        "case": "chung_lu_c_original_undirected_loops",
        "origin": "mirrors igraph_chung_lu_game(out=[1,0,2.5,2,3,2,1.5], "
        "in=NULL, loops=true, ORIGINAL): C test asserts undirected + "
        "no parallel edges (is_simple covers no-multi but allows loops).",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 0.0, 2.5, 2.0, 3.0, 2.0, 1.5],
            "in_weights": None,
            "loops": True,
            "variant": "original",
            "seed": 12_000_004,
        },
        "expected": {
            "vcount": 7,
            "directed": False,
            "is_simple": False,
            "no_multi_edges": True,
            "ecount_min": 0,
            "ecount_max": 28,
        },
    },
    {
        "case": "chung_lu_c_original_directed_no_loops",
        "origin": "mirrors igraph_chung_lu_game(out=[1,0,2.5,2,3,2,1.5], "
        "in=[2,2,2,2,0,2,2], loops=false, ORIGINAL): C test asserts "
        "directed + simple. Σw_out = Σw_in = 12.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 0.0, 2.5, 2.0, 3.0, 2.0, 1.5],
            "in_weights": [2.0, 2.0, 2.0, 2.0, 0.0, 2.0, 2.0],
            "loops": False,
            "variant": "original",
            "seed": 12_000_005,
        },
        "expected": {
            "vcount": 7,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 42,
        },
    },
    {
        "case": "chung_lu_c_maxent_undirected_no_loops",
        "origin": "mirrors igraph_chung_lu_game(out=[189,0,2.5,12,3,2,1.5], "
        "in=NULL, loops=false, MAXENT): C test asserts undirected + "
        "simple. Maxent transforms q→q/(1+q) so very large weights "
        "saturate near 1 but stay bounded — band is wide.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [189.0, 0.0, 2.5, 12.0, 3.0, 2.0, 1.5],
            "in_weights": None,
            "loops": False,
            "variant": "maxent",
            "seed": 12_000_006,
        },
        "expected": {
            "vcount": 7,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 21,
        },
    },
    {
        "case": "chung_lu_c_maxent_directed_no_loops",
        "origin": "mirrors igraph_chung_lu_game(out=[189,0,2.5,12,3,2,1.5], "
        "in=[2,2,2,2,0,200,2], loops=false, MAXENT): C test asserts "
        "directed + simple.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [189.0, 0.0, 2.5, 12.0, 3.0, 2.0, 1.5],
            "in_weights": [2.0, 2.0, 2.0, 2.0, 0.0, 200.0, 2.0],
            "loops": False,
            "variant": "maxent",
            "seed": 12_000_007,
        },
        "expected": {
            "vcount": 7,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 42,
        },
    },
    {
        "case": "chung_lu_c_nr_undirected_no_loops",
        "origin": "mirrors igraph_chung_lu_game(out=[189,0,2.5,12,3,2,1.5], "
        "in=NULL, loops=false, NR): C test asserts undirected + simple. "
        "NR transforms q→1-exp(-q); large weights saturate near 1.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [189.0, 0.0, 2.5, 12.0, 3.0, 2.0, 1.5],
            "in_weights": None,
            "loops": False,
            "variant": "nr",
            "seed": 12_000_008,
        },
        "expected": {
            "vcount": 7,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 21,
        },
    },
    {
        "case": "chung_lu_c_nr_directed_no_loops",
        "origin": "mirrors igraph_chung_lu_game(out=[189,0,2.5,12,3,2,1.5], "
        "in=[2,2,2,2,0,200,2], loops=false, NR): C test asserts "
        "directed + simple.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [189.0, 0.0, 2.5, 12.0, 3.0, 2.0, 1.5],
            "in_weights": [2.0, 2.0, 2.0, 2.0, 0.0, 200.0, 2.0],
            "loops": False,
            "variant": "nr",
            "seed": 12_000_009,
        },
        "expected": {
            "vcount": 7,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 42,
        },
    },
]


# ALGO-GN-013 (static_fitness_game). The C reference at
# references/igraph/src/games/static_fitness.c:110 has no dedicated unit
# test for this entry point — the only public C test exercises the
# power-law wrapper. Cases below are constructed to mirror the happy
# paths the C source explicitly handles: empty graph (n=0), zero-edge
# requests on positive n, all-zero fitness pinning ecount=0, and
# loops/multiple combinations with realisable demand. Our sampler is
# deterministic per seed and always reaches the requested edge count
# unless capacity is exceeded (rejected upfront), so ecount is pinned
# exactly. RNG is not portable across implementations — only structural
# invariants are asserted.
STATIC_FITNESS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "static_fitness_c_zero_vertices_undirected",
        "origin": "constructed (mirrors igraph_static_fitness_game with n=0, "
        "fitness_in=NULL): degenerate empty undirected graph.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 0,
            "fitness_out": [],
            "fitness_in": None,
            "loops": False,
            "multiple": False,
            "seed": 12_001_001,
        },
        "expected": {
            "vcount": 0,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "static_fitness_c_zero_edges_simple",
        "origin": "constructed: ten vertices, m=0 → isolated graph regardless "
        "of fitness shape.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 0,
            "fitness_out": [1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0],
            "fitness_in": None,
            "loops": False,
            "multiple": False,
            "seed": 12_001_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "static_fitness_c_undirected_simple",
        "origin": "constructed: monotone-decreasing fitness, undirected simple "
        "(loops=false, multiple=false). Capacity = C(8,2) = 28 ≥ 10.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 10,
            "fitness_out": [8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0],
            "fitness_in": None,
            "loops": False,
            "multiple": False,
            "seed": 12_001_003,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "is_simple": True,
            "ecount_min": 10,
            "ecount_max": 10,
        },
    },
    {
        "case": "static_fitness_c_undirected_loops_no_multi",
        "origin": "constructed: undirected, loops=true, multiple=false. "
        "no_multi_edges holds (rejection drops parallel pairs) but "
        "is_simple is false because self-loops are permitted.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 12,
            "fitness_out": [3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0],
            "fitness_in": None,
            "loops": True,
            "multiple": False,
            "seed": 12_001_004,
        },
        "expected": {
            "vcount": 7,
            "directed": False,
            "is_simple": False,
            "no_multi_edges": True,
            "ecount_min": 12,
            "ecount_max": 12,
        },
    },
    {
        "case": "static_fitness_c_undirected_multi_loops",
        "origin": "constructed: undirected, loops=true, multiple=true. Both "
        "self-loops and parallel edges allowed; ecount = m exactly.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 25,
            "fitness_out": [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            "fitness_in": None,
            "loops": True,
            "multiple": True,
            "seed": 12_001_005,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "ecount_min": 25,
            "ecount_max": 25,
        },
    },
    {
        "case": "static_fitness_c_directed_simple",
        "origin": "constructed: directed simple — separate fitness_in vector. "
        "Capacity = n*(n-1) = 30 (no loops) ≥ 12.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 12,
            "fitness_out": [3.0, 2.0, 1.0, 4.0, 2.0, 1.0],
            "fitness_in": [1.0, 2.0, 3.0, 1.0, 2.0, 4.0],
            "loops": False,
            "multiple": False,
            "seed": 12_001_006,
        },
        "expected": {
            "vcount": 6,
            "directed": True,
            "is_simple": True,
            "ecount_min": 12,
            "ecount_max": 12,
        },
    },
    {
        "case": "static_fitness_c_directed_multi_loops",
        "origin": "constructed: directed, loops=true, multiple=true. "
        "ecount = m exactly. Larger sample to exercise the trivial "
        "sample-and-keep branch.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 50,
            "fitness_out": [5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
            "fitness_in": [5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
            "loops": True,
            "multiple": True,
            "seed": 12_001_007,
        },
        "expected": {
            "vcount": 6,
            "directed": True,
            "ecount_min": 50,
            "ecount_max": 50,
        },
    },
    {
        "case": "static_fitness_c_single_vertex_loops_multi",
        "origin": "constructed: single vertex, loops=true, multiple=true, "
        "every edge is the (0,0) self-loop.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 5,
            "fitness_out": [1.0],
            "fitness_in": None,
            "loops": True,
            "multiple": True,
            "seed": 12_001_008,
        },
        "expected": {
            "vcount": 1,
            "directed": False,
            "ecount_min": 5,
            "ecount_max": 5,
        },
    },
]


# ALGO-GN-013 (static_power_law_game). Mirrors the eight happy-path
# tests in references/igraph/tests/unit/igraph_static_power_law_game.c.
# Each C test asserts vcount and ecount exactly. The flag combinations
# decode as:
#   IGRAPH_SIMPLE_SW   → loops=false, multiple=false
#   IGRAPH_LOOPS_SW    → loops=true,  multiple=false
#   IGRAPH_MULTI_SW    → loops=false, multiple=true
#   LOOPS_SW|MULTI_SW  → loops=true,  multiple=true
# A negative `exponent_in` in the C call selects undirected (passes
# fitness_in=NULL down the stack); a non-negative value selects
# directed. RNG is not portable, so we pin ecount_min = ecount_max = m.
STATIC_POWER_LAW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "static_power_law_c_no_vertices_directed",
        "origin": "mirrors igraph_static_power_law_game.c:28 "
        "(n=0, m=0, exp_out=2.0, exp_in=2.0, SIMPLE).",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 0,
            "no_of_edges": 0,
            "exponent_out": 2.0,
            "exponent_in": 2.0,
            "loops": False,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_002_001,
        },
        "expected": {
            "vcount": 0,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "static_power_law_c_no_edges_undirected",
        "origin": "mirrors igraph_static_power_law_game.c:35 "
        "(n=10, m=0, exp_out=2.0, exp_in=-2.0 → undirected, SIMPLE).",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 10,
            "no_of_edges": 0,
            "exponent_out": 2.0,
            "exponent_in": None,
            "loops": False,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_002_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "static_power_law_c_undirected_loops_multi",
        "origin": "mirrors igraph_static_power_law_game.c:42 "
        "(n=100, m=30, exp_out=2.0, exp_in=-2.0 → undirected, "
        "LOOPS|MULTI). C asserts vcount==100 && ecount==30.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 100,
            "no_of_edges": 30,
            "exponent_out": 2.0,
            "exponent_in": None,
            "loops": True,
            "multiple": True,
            "finite_size_correction": True,
            "seed": 12_002_003,
        },
        "expected": {
            "vcount": 100,
            "directed": False,
            "ecount_min": 30,
            "ecount_max": 30,
        },
    },
    {
        "case": "static_power_law_c_undirected_loops_only",
        "origin": "mirrors igraph_static_power_law_game.c:49 "
        "(n=90, m=40, exp_out=2.0, exp_in=-2.0 → undirected, LOOPS). "
        "loops=true, multiple=false → no_multi_edges only.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 90,
            "no_of_edges": 40,
            "exponent_out": 2.0,
            "exponent_in": None,
            "loops": True,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_002_004,
        },
        "expected": {
            "vcount": 90,
            "directed": False,
            "is_simple": False,
            "no_multi_edges": True,
            "ecount_min": 40,
            "ecount_max": 40,
        },
    },
    {
        "case": "static_power_law_c_undirected_multi_only",
        "origin": "mirrors igraph_static_power_law_game.c:56 "
        "(n=110, m=50, exp_out=2.0, exp_in=-2.0 → undirected, MULTI). "
        "loops=false, multiple=true.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 110,
            "no_of_edges": 50,
            "exponent_out": 2.0,
            "exponent_in": None,
            "loops": False,
            "multiple": True,
            "finite_size_correction": True,
            "seed": 12_002_005,
        },
        "expected": {
            "vcount": 110,
            "directed": False,
            "ecount_min": 50,
            "ecount_max": 50,
        },
    },
    {
        "case": "static_power_law_c_directed_loops_multi",
        "origin": "mirrors igraph_static_power_law_game.c:63 "
        "(n=100, m=30, exp_out=2.0, exp_in=2.0, LOOPS|MULTI). "
        "Directed.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 100,
            "no_of_edges": 30,
            "exponent_out": 2.0,
            "exponent_in": 2.0,
            "loops": True,
            "multiple": True,
            "finite_size_correction": True,
            "seed": 12_002_006,
        },
        "expected": {
            "vcount": 100,
            "directed": True,
            "ecount_min": 30,
            "ecount_max": 30,
        },
    },
    {
        "case": "static_power_law_c_directed_loops_only",
        "origin": "mirrors igraph_static_power_law_game.c:70 "
        "(n=90, m=40, exp_out=2.0, exp_in=2.2, LOOPS). Directed, "
        "loops=true, multiple=false.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 90,
            "no_of_edges": 40,
            "exponent_out": 2.0,
            "exponent_in": 2.2,
            "loops": True,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_002_007,
        },
        "expected": {
            "vcount": 90,
            "directed": True,
            "is_simple": False,
            "no_multi_edges": True,
            "ecount_min": 40,
            "ecount_max": 40,
        },
    },
    {
        "case": "static_power_law_c_directed_multi_only",
        "origin": "mirrors igraph_static_power_law_game.c:77 "
        "(n=110, m=50, exp_out=2.0, exp_in=2.5, MULTI). Directed, "
        "loops=false, multiple=true.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 110,
            "no_of_edges": 50,
            "exponent_out": 2.0,
            "exponent_in": 2.5,
            "loops": False,
            "multiple": True,
            "finite_size_correction": True,
            "seed": 12_002_008,
        },
        "expected": {
            "vcount": 110,
            "directed": True,
            "ecount_min": 50,
            "ecount_max": 50,
        },
    },
]


# ALGO-CN-001: ring (igraph_ring + path_graph + cycle_graph wrappers).
# Mirrors `igraph_ring` in `constructors/regular.c:495-604`. Fully
# deterministic — no RNG — so `expected.edges` is exact (raw upstream
# enumeration order: forward arcs (i, i+1), back-arcs (i+1, i) when
# `directed && mutual`, then `(n-1, 0)` wrap when `circular`, plus
# `(0, n-1)` mutual wrap). Rust storage canonicalises undirected edges
# (min endpoint first), so the harness must compare via multisets of
# canonicalised tuples for undirected fixtures and exact-ordered vectors
# for directed fixtures.
RING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ring_c_path_p5_undirected",
        "origin": "mirrors igraph_ring(n=5, directed=0, mutual=0, "
        "circular=0) — open path P5",
        "algo": "ring_graph",
        "params": {"n": 5, "directed": False, "mutual": False, "circular": False},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4]],
        },
    },
    {
        "case": "ring_c_cycle_c5_undirected",
        "origin": "mirrors igraph_ring(n=5, directed=0, mutual=0, "
        "circular=1) — cycle C5 (raw order includes (4,0) wrap)",
        "algo": "ring_graph",
        "params": {"n": 5, "directed": False, "mutual": False, "circular": True},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [4, 0]],
        },
    },
    {
        "case": "ring_c_path_p4_directed",
        "origin": "mirrors igraph_ring(n=4, directed=1, mutual=0, "
        "circular=0) — directed forward path",
        "algo": "ring_graph",
        "params": {"n": 4, "directed": True, "mutual": False, "circular": False},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": True,
            "edges": [[0, 1], [1, 2], [2, 3]],
        },
    },
    {
        "case": "ring_c_cycle_c4_directed_mutual",
        "origin": "mirrors igraph_ring(n=4, directed=1, mutual=1, "
        "circular=1) — every link emits both arcs + wrap back-arc last",
        "algo": "ring_graph",
        "params": {"n": 4, "directed": True, "mutual": True, "circular": True},
        "expected": {
            "vcount": 4,
            "ecount": 8,
            "directed": True,
            "edges": [
                [0, 1], [1, 0], [1, 2], [2, 1],
                [2, 3], [3, 2], [3, 0], [0, 3],
            ],
        },
    },
    {
        "case": "ring_c_singleton_self_loop",
        "origin": "mirrors igraph_ring(n=1, directed=0, mutual=0, "
        "circular=1) — degenerate cycle becomes self-loop (0,0)",
        "algo": "ring_graph",
        "params": {"n": 1, "directed": False, "mutual": False, "circular": True},
        "expected": {
            "vcount": 1,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 0]],
        },
    },
    {
        "case": "ring_c_empty",
        "origin": "mirrors igraph_ring(n=0, ...) — empty graph regardless of flags",
        "algo": "ring_graph",
        "params": {"n": 0, "directed": False, "mutual": False, "circular": False},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
]


# Hand-derived from `igraph_star()` in src/constructors/regular.c:75-141.
# The C entry point allocates `2(n-1)` edge slots (`4(n-1)` for MUTUAL),
# walks leaves in raw vertex-id order `[0, center) ∪ (center, n)`, and
# emits a `center → leaf` arc for OUT, `leaf → center` for IN/UNDIRECTED,
# and both arcs (forward first) for MUTUAL. Empty graph when n == 0;
# single-vertex graph has no edges. Rust storage canonicalises undirected
# edges (min endpoint first), so the harness compares via multisets of
# canonicalised tuples for undirected fixtures and exact-ordered vectors
# for directed fixtures.
STAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "star_c_undirected_k1_4",
        "origin": "mirrors igraph_star(n=5, mode=UNDIRECTED, center=0) — "
        "K1,4 with vertex 0 as the centre",
        "algo": "star_graph",
        "params": {"n": 5, "mode": "Undirected", "center": 0},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": False,
            "edges": [[1, 0], [2, 0], [3, 0], [4, 0]],
        },
    },
    {
        "case": "star_c_out_center_zero",
        "origin": "mirrors igraph_star(n=5, mode=OUT, center=0) — "
        "directed out-star, centre emits to every leaf",
        "algo": "star_graph",
        "params": {"n": 5, "mode": "Out", "center": 0},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 1], [0, 2], [0, 3], [0, 4]],
        },
    },
    {
        "case": "star_c_in_center_zero",
        "origin": "mirrors igraph_star(n=5, mode=IN, center=0) — "
        "directed in-star, every leaf emits to centre",
        "algo": "star_graph",
        "params": {"n": 5, "mode": "In", "center": 0},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": True,
            "edges": [[1, 0], [2, 0], [3, 0], [4, 0]],
        },
    },
    {
        "case": "star_c_mutual_center_zero",
        "origin": "mirrors igraph_star(n=4, mode=MUTUAL, center=0) — "
        "both arcs per leaf, forward arc first per upstream loop",
        "algo": "star_graph",
        "params": {"n": 4, "mode": "Mutual", "center": 0},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": True,
            "edges": [
                [0, 1], [1, 0], [0, 2], [2, 0], [0, 3], [3, 0],
            ],
        },
    },
    {
        "case": "star_c_out_center_two",
        "origin": "mirrors igraph_star(n=5, mode=OUT, center=2) — "
        "leaves visited in raw vertex-id order [0,1] then [3,4]",
        "algo": "star_graph",
        "params": {"n": 5, "mode": "Out", "center": 2},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": True,
            "edges": [[2, 0], [2, 1], [2, 3], [2, 4]],
        },
    },
    {
        "case": "star_c_empty",
        "origin": "mirrors igraph_star(n=0, ...) — empty graph regardless of mode",
        "algo": "star_graph",
        "params": {"n": 0, "mode": "Out", "center": 0},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]


WHEEL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "wheel_c_undirected_w6",
        "origin": "mirrors igraph_wheel(n=6, mode=UNDIRECTED, center=0) — "
        "5 spokes + 5 rim edges; centre degree 5, rim degree 3",
        "algo": "wheel_graph",
        "params": {"n": 6, "mode": "Undirected", "center": 0},
        "expected": {
            "vcount": 6,
            "ecount": 10,
            "directed": False,
            "edges": [
                [1, 0], [2, 0], [3, 0], [4, 0], [5, 0],
                [1, 2], [2, 3], [3, 4], [4, 5], [5, 1],
            ],
        },
    },
    {
        "case": "wheel_c_out_w5_center_zero",
        "origin": "mirrors igraph_wheel(n=5, mode=OUT, center=0) — "
        "directed wheel, all spokes and rim arcs flow forward",
        "algo": "wheel_graph",
        "params": {"n": 5, "mode": "Out", "center": 0},
        "expected": {
            "vcount": 5,
            "ecount": 8,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4],
                [1, 2], [2, 3], [3, 4], [4, 1],
            ],
        },
    },
    {
        "case": "wheel_c_in_w5_center_zero",
        "origin": "mirrors igraph_wheel(n=5, mode=IN, center=0) — "
        "directed wheel, spokes leaf→centre, rim still prev→next",
        "algo": "wheel_graph",
        "params": {"n": 5, "mode": "In", "center": 0},
        "expected": {
            "vcount": 5,
            "ecount": 8,
            "directed": True,
            "edges": [
                [1, 0], [2, 0], [3, 0], [4, 0],
                [1, 2], [2, 3], [3, 4], [4, 1],
            ],
        },
    },
    {
        "case": "wheel_c_mutual_w4_center_zero",
        "origin": "mirrors igraph_wheel(n=4, mode=MUTUAL, center=0) — "
        "spokes mutual then rim forward followed by reverse-discovery",
        "algo": "wheel_graph",
        "params": {"n": 4, "mode": "Mutual", "center": 0},
        "expected": {
            "vcount": 4,
            "ecount": 12,
            "directed": True,
            "edges": [
                [0, 1], [1, 0], [0, 2], [2, 0], [0, 3], [3, 0],
                [1, 2], [2, 3], [3, 1],
                [1, 3], [3, 2], [2, 1],
            ],
        },
    },
    {
        "case": "wheel_c_out_w5_center_two",
        "origin": "mirrors igraph_wheel(n=5, mode=OUT, center=2) — "
        "rim skips the centre, visits leaves in raw vertex-id order",
        "algo": "wheel_graph",
        "params": {"n": 5, "mode": "Out", "center": 2},
        "expected": {
            "vcount": 5,
            "ecount": 8,
            "directed": True,
            "edges": [
                [2, 0], [2, 1], [2, 3], [2, 4],
                [0, 1], [1, 3], [3, 4], [4, 0],
            ],
        },
    },
    {
        "case": "wheel_c_empty",
        "origin": "mirrors igraph_wheel(n=0, ...) — empty graph regardless of mode",
        "algo": "wheel_graph",
        "params": {"n": 0, "mode": "Out", "center": 0},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]

KARY_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "kary_tree_c_binary_seven_out",
        "origin": "mirrors igraph_kary_tree(n=7, children=2, IGRAPH_TREE_OUT) — "
        "perfect binary tree depth 2, parent→child arcs",
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
        "case": "kary_tree_c_binary_seven_in",
        "origin": "mirrors igraph_kary_tree(n=7, children=2, IGRAPH_TREE_IN) — "
        "perfect binary tree depth 2, child→parent arcs",
        "algo": "kary_tree",
        "params": {"n": 7, "children": 2, "mode": "In"},
        "expected": {
            "vcount": 7,
            "ecount": 6,
            "directed": True,
            "edges": [
                [1, 0], [2, 0], [3, 1], [4, 1], [5, 2], [6, 2],
            ],
        },
    },
    {
        "case": "kary_tree_c_binary_seven_undirected",
        "origin": "mirrors igraph_kary_tree(n=7, children=2, IGRAPH_TREE_UNDIRECTED) — "
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
        "case": "kary_tree_c_ternary_eight_partial",
        "origin": "mirrors igraph_kary_tree(n=8, children=3, IGRAPH_TREE_OUT) — "
        "last parent gets only one child (8-1=7 edges, not multiple of 3)",
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
        "case": "kary_tree_c_chain_one_child",
        "origin": "mirrors igraph_kary_tree(n=6, children=1, IGRAPH_TREE_OUT) — "
        "linear chain (path) of 6 vertices",
        "algo": "kary_tree",
        "params": {"n": 6, "children": 1, "mode": "Out"},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "directed": True,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5],
            ],
        },
    },
    {
        "case": "kary_tree_c_empty",
        "origin": "mirrors igraph_kary_tree(n=0, ...) — empty graph regardless of children/mode",
        "algo": "kary_tree",
        "params": {"n": 0, "children": 2, "mode": "Out"},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]

SYMMETRIC_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "symmetric_tree_c_binary_depth_two_out",
        "origin": "mirrors igraph_symmetric_tree(branches=[2,2], IGRAPH_TREE_OUT) — "
        "1 + 2 + 4 = 7 vertices, identical to kary_tree(7, 2)",
        "algo": "symmetric_tree",
        "params": {"branches": [2, 2], "mode": "Out"},
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
        "case": "symmetric_tree_c_binary_depth_two_in",
        "origin": "mirrors igraph_symmetric_tree(branches=[2,2], IGRAPH_TREE_IN) — "
        "child→parent arcs",
        "algo": "symmetric_tree",
        "params": {"branches": [2, 2], "mode": "In"},
        "expected": {
            "vcount": 7,
            "ecount": 6,
            "directed": True,
            "edges": [
                [1, 0], [2, 0], [3, 1], [4, 1], [5, 2], [6, 2],
            ],
        },
    },
    {
        "case": "symmetric_tree_c_three_then_two_undirected",
        "origin": "mirrors igraph_symmetric_tree(branches=[3,2], IGRAPH_TREE_UNDIRECTED) — "
        "1 + 3 + 6 = 10 vertices, undirected",
        "algo": "symmetric_tree",
        "params": {"branches": [3, 2], "mode": "Undirected"},
        "expected": {
            "vcount": 10,
            "ecount": 9,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 4], [1, 5], [2, 6], [2, 7], [3, 8], [3, 9],
            ],
        },
    },
    {
        "case": "symmetric_tree_c_three_levels_three_two_one_out",
        "origin": "mirrors igraph_symmetric_tree(branches=[3,2,1], IGRAPH_TREE_OUT) — "
        "1 + 3 + 6 + 6 = 16 vertices, depth 3",
        "algo": "symmetric_tree",
        "params": {"branches": [3, 2, 1], "mode": "Out"},
        "expected": {
            "vcount": 16,
            "ecount": 15,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 4], [1, 5], [2, 6], [2, 7], [3, 8], [3, 9],
                [4, 10], [5, 11], [6, 12], [7, 13], [8, 14], [9, 15],
            ],
        },
    },
    {
        "case": "symmetric_tree_c_single_leaf_branches_one_out",
        "origin": "mirrors igraph_symmetric_tree(branches=[1], IGRAPH_TREE_OUT) — "
        "root + single child, 2 vertices",
        "algo": "symmetric_tree",
        "params": {"branches": [1], "mode": "Out"},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": True,
            "edges": [[0, 1]],
        },
    },
    {
        "case": "symmetric_tree_c_empty_branches_singleton",
        "origin": "mirrors igraph_symmetric_tree(branches=[], IGRAPH_TREE_OUT) — "
        "empty branches collapses to singleton root",
        "algo": "symmetric_tree",
        "params": {"branches": [], "mode": "Out"},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]


REGULAR_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "regular_tree_c_h1_k3_out",
        "origin": "mirrors igraph_regular_tree(h=1, k=3, IGRAPH_TREE_OUT) — "
        "root with 3 leaves (star K1,3, equivalent to symmetric_tree([3]))",
        "algo": "regular_tree",
        "params": {"h": 1, "k": 3, "mode": "Out"},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3],
            ],
        },
    },
    {
        "case": "regular_tree_c_h2_k3_out",
        "origin": "mirrors igraph_regular_tree(h=2, k=3, IGRAPH_TREE_OUT) — "
        "Bethe lattice with root deg 3, each internal deg 3 (branches=[3,2])",
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
        "case": "regular_tree_c_h2_k3_in",
        "origin": "mirrors igraph_regular_tree(h=2, k=3, IGRAPH_TREE_IN) — "
        "child→parent arcs",
        "algo": "regular_tree",
        "params": {"h": 2, "k": 3, "mode": "In"},
        "expected": {
            "vcount": 10,
            "ecount": 9,
            "directed": True,
            "edges": [
                [1, 0], [2, 0], [3, 0],
                [4, 1], [5, 1], [6, 2], [7, 2], [8, 3], [9, 3],
            ],
        },
    },
    {
        "case": "regular_tree_c_h2_k3_undirected",
        "origin": "mirrors igraph_regular_tree(h=2, k=3, IGRAPH_TREE_UNDIRECTED) — "
        "undirected Bethe lattice",
        "algo": "regular_tree",
        "params": {"h": 2, "k": 3, "mode": "Undirected"},
        "expected": {
            "vcount": 10,
            "ecount": 9,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 4], [1, 5], [2, 6], [2, 7], [3, 8], [3, 9],
            ],
        },
    },
    {
        "case": "regular_tree_c_h3_k2_out",
        "origin": "mirrors igraph_regular_tree(h=3, k=2, IGRAPH_TREE_OUT) — "
        "degenerate k=2 case (branches=[2,1,1]) — a tree of height 3",
        "algo": "regular_tree",
        "params": {"h": 3, "k": 2, "mode": "Out"},
        "expected": {
            "vcount": 7,
            "ecount": 6,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [1, 3], [2, 4], [3, 5], [4, 6],
            ],
        },
    },
    {
        "case": "regular_tree_c_h2_k4_out",
        "origin": "mirrors igraph_regular_tree(h=2, k=4, IGRAPH_TREE_OUT) — "
        "root deg 4, each internal deg 4 (branches=[4,3])",
        "algo": "regular_tree",
        "params": {"h": 2, "k": 4, "mode": "Out"},
        "expected": {
            "vcount": 17,
            "ecount": 16,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4],
                [1, 5], [1, 6], [1, 7],
                [2, 8], [2, 9], [2, 10],
                [3, 11], [3, 12], [3, 13],
                [4, 14], [4, 15], [4, 16],
            ],
        },
    },
]


HYPERCUBE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hypercube_c_n0_undirected",
        "origin": "mirrors igraph_hypercube(n=0, directed=false) — degenerate singleton",
        "algo": "hypercube",
        "params": {"n": 0, "directed": False},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "hypercube_c_n1_undirected",
        "origin": "mirrors igraph_hypercube(n=1, directed=false) — Q_1 = K_2",
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
        "case": "hypercube_c_n2_undirected",
        "origin": "mirrors igraph_hypercube(n=2, directed=false) — Q_2 is the 4-cycle",
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
        "case": "hypercube_c_n3_undirected",
        "origin": "mirrors igraph_hypercube(n=3, directed=false) — 8-vertex cube Q_3",
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
        "case": "hypercube_c_n3_directed",
        "origin": "mirrors igraph_hypercube(n=3, directed=true) — Q_3 oriented low->high",
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
    {
        "case": "hypercube_c_n4_undirected",
        "origin": "mirrors igraph_hypercube(n=4, directed=false) — Q_4 with 16 vertices, 32 edges",
        "algo": "hypercube",
        "params": {"n": 4, "directed": False},
        "expected": {
            "vcount": 16,
            "ecount": 32,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 4], [0, 8],
                [1, 3], [1, 5], [1, 9],
                [2, 3], [2, 6], [2, 10],
                [3, 7], [3, 11],
                [4, 5], [4, 6], [4, 12],
                [5, 7], [5, 13],
                [6, 7], [6, 14],
                [7, 15],
                [8, 9], [8, 10], [8, 12],
                [9, 11], [9, 13],
                [10, 11], [10, 14],
                [11, 15],
                [12, 13], [12, 14],
                [13, 15],
                [14, 15],
            ],
        },
    },
]


SQUARE_LATTICE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "square_lattice_c_dim_zero_singleton",
        "origin": "mirrors igraph_square_lattice(dim=[], nei=1) — empty-dim singleton",
        "algo": "square_lattice",
        "params": {
            "dim": [],
            "nei": 1,
            "directed": False,
            "mutual": False,
            "periodic": None,
        },
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "square_lattice_c_dim_three_path",
        "origin": "mirrors igraph_square_lattice(dim=[3], nei=1, periodic=[false]) — path P_3",
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
        "case": "square_lattice_c_dim_three_periodic_cycle",
        "origin": "mirrors igraph_square_lattice(dim=[3], nei=1, periodic=[true]) — cycle C_3",
        "algo": "square_lattice",
        "params": {
            "dim": [3],
            "nei": 1,
            "directed": False,
            "mutual": False,
            "periodic": [True],
        },
        "expected": {
            "vcount": 3,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [1, 2], [0, 2]],
        },
    },
    {
        "case": "square_lattice_c_dim_2x2_four_cycle",
        "origin": "mirrors igraph_square_lattice(dim=[2,2], nei=1) — 2x2 grid is the 4-cycle",
        "algo": "square_lattice",
        "params": {
            "dim": [2, 2],
            "nei": 1,
            "directed": False,
            "mutual": False,
            "periodic": None,
        },
        "expected": {
            "vcount": 4,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 3], [2, 3]],
        },
    },
    {
        "case": "square_lattice_c_dim_3x3_grid",
        "origin": "mirrors igraph_square_lattice(dim=[3,3], nei=1) — 3x3 grid, 9 v 12 e",
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
        "case": "square_lattice_c_dim_3x3_torus",
        "origin": "mirrors igraph_square_lattice(dim=[3,3], periodic=[true,true]) — 3x3 torus, 18 e 4-regular",
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
        "case": "square_lattice_c_dim_2x2x2_cube",
        "origin": "mirrors igraph_square_lattice(dim=[2,2,2], nei=1) — Q_3 cube, 8 v 12 e",
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
    {
        "case": "square_lattice_c_dim_three_directed_mutual",
        "origin": "mirrors igraph_square_lattice(dim=[3], directed=true, mutual=true) — arcs both ways on path",
        "algo": "square_lattice",
        "params": {
            "dim": [3],
            "nei": 1,
            "directed": True,
            "mutual": True,
            "periodic": None,
        },
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 1], [1, 2], [1, 0], [2, 1]],
        },
    },
]


HAMMING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hamming_c_n0_q5_singleton",
        "origin": "mirrors igraph_hamming(n=0, q=5, directed=false) — degenerate singleton",
        "algo": "hamming",
        "params": {"n": 0, "q": 5, "directed": False},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "hamming_c_n2_q0_null_graph",
        "origin": "mirrors igraph_hamming(n=2, q=0, directed=false) — empty alphabet, null graph",
        "algo": "hamming",
        "params": {"n": 2, "q": 0, "directed": False},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "hamming_c_n1_q3_is_k3",
        "origin": "mirrors igraph_hamming(n=1, q=3, directed=false) — H(1,3) = K_3",
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
        "case": "hamming_c_n2_q3_undirected",
        "origin": "mirrors igraph_hamming(n=2, q=3, directed=false) — H(2,3), 9 vertices, 18 edges",
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
        "case": "hamming_c_n3_q2_is_hypercube_q3",
        "origin": "mirrors igraph_hamming(n=3, q=2, directed=false) — H(3,2) ≡ Q_3",
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
        "case": "hamming_c_n2_q4_undirected",
        "origin": "mirrors igraph_hamming(n=2, q=4, directed=false) — H(2,4), 16 vertices, 48 edges",
        "algo": "hamming",
        "params": {"n": 2, "q": 4, "directed": False},
        "expected": {
            "vcount": 16,
            "ecount": 48,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4], [0, 8], [0, 12],
                [1, 2], [1, 3], [1, 5], [1, 9], [1, 13],
                [2, 3], [2, 6], [2, 10], [2, 14],
                [3, 7], [3, 11], [3, 15],
                [4, 5], [4, 6], [4, 7], [4, 8], [4, 12],
                [5, 6], [5, 7], [5, 9], [5, 13],
                [6, 7], [6, 10], [6, 14],
                [7, 11], [7, 15],
                [8, 9], [8, 10], [8, 11], [8, 12],
                [9, 10], [9, 11], [9, 13],
                [10, 11], [10, 14],
                [11, 15],
                [12, 13], [12, 14], [12, 15],
                [13, 14], [13, 15],
                [14, 15],
            ],
        },
    },
]


GENERALIZED_PETERSEN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "generalized_petersen_c_g_3_1",
        "origin": "mirrors igraph_generalized_petersen(n=3, k=1) — triangular prism, smallest valid",
        "algo": "generalized_petersen",
        "params": {"n": 3, "k": 1},
        "expected": {
            "vcount": 6,
            "ecount": 9,
            "directed": False,
            "edges": [
                [0, 1], [0, 3], [3, 4],
                [1, 2], [1, 4], [4, 5],
                [0, 2], [2, 5], [3, 5],
            ],
        },
    },
    {
        "case": "generalized_petersen_c_g_4_1",
        "origin": "mirrors igraph_generalized_petersen(n=4, k=1) — 4-prism ≡ Q_3 (3-cube)",
        "algo": "generalized_petersen",
        "params": {"n": 4, "k": 1},
        "expected": {
            "vcount": 8,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 4], [4, 5],
                [1, 2], [1, 5], [5, 6],
                [2, 3], [2, 6], [6, 7],
                [0, 3], [3, 7], [4, 7],
            ],
        },
    },
    {
        "case": "generalized_petersen_c_g_5_2_petersen",
        "origin": "mirrors igraph_generalized_petersen(n=5, k=2) — the classic Petersen graph",
        "algo": "generalized_petersen",
        "params": {"n": 5, "k": 2},
        "expected": {
            "vcount": 10,
            "ecount": 15,
            "directed": False,
            "edges": [
                [0, 1], [0, 5], [5, 7],
                [1, 2], [1, 6], [6, 8],
                [2, 3], [2, 7], [7, 9],
                [3, 4], [3, 8], [5, 8],
                [0, 4], [4, 9], [6, 9],
            ],
        },
    },
    {
        "case": "generalized_petersen_c_g_6_2",
        "origin": "mirrors igraph_generalized_petersen(n=6, k=2) — even-n with non-trivial circulant shift",
        "algo": "generalized_petersen",
        "params": {"n": 6, "k": 2},
        "expected": {
            "vcount": 12,
            "ecount": 18,
            "directed": False,
            "edges": [
                [0, 1], [0, 6], [6, 8],
                [1, 2], [1, 7], [7, 9],
                [2, 3], [2, 8], [8, 10],
                [3, 4], [3, 9], [9, 11],
                [4, 5], [4, 10], [6, 10],
                [0, 5], [5, 11], [7, 11],
            ],
        },
    },
    {
        "case": "generalized_petersen_c_g_7_2",
        "origin": "mirrors igraph_generalized_petersen(n=7, k=2) — odd-n with k=2",
        "algo": "generalized_petersen",
        "params": {"n": 7, "k": 2},
        "expected": {
            "vcount": 14,
            "ecount": 21,
            "directed": False,
            "edges": [
                [0, 1], [0, 7], [7, 9],
                [1, 2], [1, 8], [8, 10],
                [2, 3], [2, 9], [9, 11],
                [3, 4], [3, 10], [10, 12],
                [4, 5], [4, 11], [11, 13],
                [5, 6], [5, 12], [7, 12],
                [0, 6], [6, 13], [8, 13],
            ],
        },
    },
    {
        "case": "generalized_petersen_c_g_8_3_mobius_kantor",
        "origin": "mirrors igraph_generalized_petersen(n=8, k=3) — the Möbius–Kantor graph",
        "algo": "generalized_petersen",
        "params": {"n": 8, "k": 3},
        "expected": {
            "vcount": 16,
            "ecount": 24,
            "directed": False,
            "edges": [
                [0, 1], [0, 8], [8, 11],
                [1, 2], [1, 9], [9, 12],
                [2, 3], [2, 10], [10, 13],
                [3, 4], [3, 11], [11, 14],
                [4, 5], [4, 12], [12, 15],
                [5, 6], [5, 13], [8, 13],
                [6, 7], [6, 14], [9, 14],
                [0, 7], [7, 15], [10, 15],
            ],
        },
    },
]


CIRCULANT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "circulant_c_c_5_shifts_1",
        "origin": "mirrors igraph_circulant(n=5, shifts=[1], directed=false) — equivalent to C_5",
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
        "case": "circulant_c_c_6_shifts_1_3_antipodal",
        "origin": "mirrors igraph_circulant(n=6, shifts=[1,3], directed=false) — even-n with antipodal shift halves to perfect matching",
        "algo": "circulant",
        "params": {"n": 6, "shifts": [1, 3], "directed": False},
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
    {
        "case": "circulant_c_c_7_shifts_1_2_squared_cycle",
        "origin": "mirrors igraph_circulant(n=7, shifts=[1,2], directed=false) — squared cycle on 7 vertices",
        "algo": "circulant",
        "params": {"n": 7, "shifts": [1, 2], "directed": False},
        "expected": {
            "vcount": 7,
            "ecount": 14,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [0, 6],
                [0, 2], [1, 3], [2, 4], [3, 5], [4, 6], [0, 5], [1, 6],
            ],
        },
    },
    {
        "case": "circulant_c_k4_shifts_1_2_complete",
        "origin": "mirrors igraph_circulant(n=4, shifts=[1,2], directed=false) — equivalent to K_4 (every distinct undirected shift)",
        "algo": "circulant",
        "params": {"n": 4, "shifts": [1, 2], "directed": False},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [0, 3], [0, 2], [1, 3],
            ],
        },
    },
    {
        "case": "circulant_c_c_5_shifts_neg1_directed",
        "origin": "mirrors igraph_circulant(n=5, shifts=[-1], directed=true) — directed backward cycle via negative shift",
        "algo": "circulant",
        "params": {"n": 5, "shifts": [-1], "directed": True},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": True,
            "edges": [
                [0, 4], [1, 0], [2, 1], [3, 2], [4, 3],
            ],
        },
    },
    {
        "case": "circulant_c_c_8_shifts_1_3_directed",
        "origin": "mirrors igraph_circulant(n=8, shifts=[1,3], directed=true) — directed circulant with two distinct shifts",
        "algo": "circulant",
        "params": {"n": 8, "shifts": [1, 3], "directed": True},
        "expected": {
            "vcount": 8,
            "ecount": 16,
            "directed": True,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 0],
                [0, 3], [1, 4], [2, 5], [3, 6], [4, 7], [5, 0], [6, 1], [7, 2],
            ],
        },
    },
]


DE_BRUIJN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "de_bruijn_c_b_1_1_singleton_self_loop",
        "origin": "mirrors igraph_de_bruijn(m=1, n=1) — single vertex with one self-loop",
        "algo": "de_bruijn",
        "params": {"m": 1, "n": 1},
        "expected": {
            "vcount": 1,
            "ecount": 1,
            "directed": True,
            "edges": [
                [0, 0],
            ],
        },
    },
    {
        "case": "de_bruijn_c_b_2_1_directed_k2_with_loops",
        "origin": "mirrors igraph_de_bruijn(m=2, n=1) — 2 vertices, 4 directed arcs incl. both self-loops",
        "algo": "de_bruijn",
        "params": {"m": 2, "n": 1},
        "expected": {
            "vcount": 2,
            "ecount": 4,
            "directed": True,
            "edges": [
                [0, 0], [0, 1], [1, 0], [1, 1],
            ],
        },
    },
    {
        "case": "de_bruijn_c_b_2_2_canonical",
        "origin": "mirrors igraph_de_bruijn(m=2, n=2) — 4 vertices, 8 arcs via rewrite (i, (i*m mod 4) + b)",
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
        "case": "de_bruijn_c_b_3_2_canonical",
        "origin": "mirrors igraph_de_bruijn(m=3, n=2) — 9 vertices, 27 arcs (alphabet of 3, strings of length 2)",
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
    {
        "case": "de_bruijn_c_b_2_3_canonical",
        "origin": "mirrors igraph_de_bruijn(m=2, n=3) — binary length-3 strings, 8 vertices and 16 arcs",
        "algo": "de_bruijn",
        "params": {"m": 2, "n": 3},
        "expected": {
            "vcount": 8,
            "ecount": 16,
            "directed": True,
            "edges": [
                [0, 0], [0, 1],
                [1, 2], [1, 3],
                [2, 4], [2, 5],
                [3, 6], [3, 7],
                [4, 0], [4, 1],
                [5, 2], [5, 3],
                [6, 4], [6, 5],
                [7, 6], [7, 7],
            ],
        },
    },
    {
        "case": "de_bruijn_c_n_zero_singleton",
        "origin": "mirrors igraph_de_bruijn(m=k, n=0) — exactly one vertex (the empty string), zero arcs",
        "algo": "de_bruijn",
        "params": {"m": 5, "n": 0},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]


# `igraph_kautz` directed Kautz graph K(m, n) on (m+1)·m^n vertices, m·vcount arcs.
# Fixtures mirror the upstream unit test `tests/unit/igraph_kautz.c` (K(2,1), K(0,10),
# K(0,0), K(5,0)) and add larger canonical cases K(3,1), K(2,2), K(3,2) to catch
# index1/index2 / cursor / basis bugs. Edge lists generated by running
# `igraph_kautz` via python-igraph and verified to match this crate byte-for-byte.
KAUTZ_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "kautz_c_m2_n1_canonical",
        "origin": "mirrors igraph_kautz(m=2, n=1) — 6 vertices, 12 arcs (the test_kautz.c canonical case)",
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
        "case": "kautz_c_m0_n10_empty",
        "origin": "mirrors igraph_kautz(m=0, n=10) — empty 0-vertex directed graph (degenerate m=0)",
        "algo": "kautz",
        "params": {"m": 0, "n": 10},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
    {
        "case": "kautz_c_m0_n0_singleton",
        "origin": "mirrors igraph_kautz(m=0, n=0) — single vertex via the n=0 → directed K_{m+1} path",
        "algo": "kautz",
        "params": {"m": 0, "n": 0},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
    {
        "case": "kautz_c_m5_n0_directed_k6",
        "origin": "mirrors igraph_kautz(m=5, n=0) — directed K_6 with no self-loops, 30 arcs",
        "algo": "kautz",
        "params": {"m": 5, "n": 0},
        "expected": {
            "vcount": 6,
            "ecount": 30,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4], [0, 5],
                [1, 0], [1, 2], [1, 3], [1, 4], [1, 5],
                [2, 0], [2, 1], [2, 3], [2, 4], [2, 5],
                [3, 0], [3, 1], [3, 2], [3, 4], [3, 5],
                [4, 0], [4, 1], [4, 2], [4, 3], [4, 5],
                [5, 0], [5, 1], [5, 2], [5, 3], [5, 4],
            ],
        },
    },
    {
        "case": "kautz_c_m3_n1_canonical",
        "origin": "mirrors igraph_kautz(m=3, n=1) — 12 vertices, 36 arcs (alphabet of 4, length-2 strings)",
        "algo": "kautz",
        "params": {"m": 3, "n": 1},
        "expected": {
            "vcount": 12,
            "ecount": 36,
            "directed": True,
            "edges": [
                [0, 3], [0, 4], [0, 5],
                [1, 6], [1, 7], [1, 8],
                [2, 9], [2, 10], [2, 11],
                [3, 0], [3, 1], [3, 2],
                [4, 6], [4, 7], [4, 8],
                [5, 9], [5, 10], [5, 11],
                [6, 0], [6, 1], [6, 2],
                [7, 3], [7, 4], [7, 5],
                [8, 9], [8, 10], [8, 11],
                [9, 0], [9, 1], [9, 2],
                [10, 3], [10, 4], [10, 5],
                [11, 6], [11, 7], [11, 8],
            ],
        },
    },
    {
        "case": "kautz_c_m2_n2_canonical",
        "origin": "mirrors igraph_kautz(m=2, n=2) — 12 vertices, 24 arcs (alphabet of 3, length-3 strings)",
        "algo": "kautz",
        "params": {"m": 2, "n": 2},
        "expected": {
            "vcount": 12,
            "ecount": 24,
            "directed": True,
            "edges": [
                [0, 4], [0, 5],
                [1, 6], [1, 7],
                [2, 8], [2, 9],
                [3, 10], [3, 11],
                [4, 0], [4, 1],
                [5, 2], [5, 3],
                [6, 8], [6, 9],
                [7, 10], [7, 11],
                [8, 0], [8, 1],
                [9, 2], [9, 3],
                [10, 4], [10, 5],
                [11, 6], [11, 7],
            ],
        },
    },
    {
        "case": "kautz_c_m3_n2_canonical",
        "origin": "mirrors igraph_kautz(m=3, n=2) — 36 vertices, 108 arcs (alphabet of 4, length-3 strings)",
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


FULL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_c_n0_null",
        "origin": "mirrors igraph_full(n=0, directed=false, loops=false) — empty graph (Null variant from tests/unit/full.c)",
        "algo": "full_graph",
        "params": {"n": 0, "directed": False, "loops": False},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "full_c_n1_singleton_noloops",
        "origin": "mirrors igraph_full(n=1, directed=false, loops=false) — Singleton, no loops",
        "algo": "full_graph",
        "params": {"n": 1, "directed": False, "loops": False},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "full_c_n1_singleton_loops",
        "origin": "mirrors igraph_full(n=1, directed=false, loops=true) — Singleton, with loops (a single self-loop)",
        "algo": "full_graph",
        "params": {"n": 1, "directed": False, "loops": True},
        "expected": {
            "vcount": 1,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 0]],
        },
    },
    {
        "case": "full_c_n10_ud_noloops",
        "origin": "mirrors igraph_full(n=10, directed=false, loops=false) — undirected K_10, 45 edges (Undirected, no loops case from tests/unit/full.out)",
        "algo": "full_graph",
        "params": {"n": 10, "directed": False, "loops": False},
        "expected": {
            "vcount": 10,
            "ecount": 45,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4], [0, 5], [0, 6], [0, 7], [0, 8], [0, 9],
                [1, 2], [1, 3], [1, 4], [1, 5], [1, 6], [1, 7], [1, 8], [1, 9],
                [2, 3], [2, 4], [2, 5], [2, 6], [2, 7], [2, 8], [2, 9],
                [3, 4], [3, 5], [3, 6], [3, 7], [3, 8], [3, 9],
                [4, 5], [4, 6], [4, 7], [4, 8], [4, 9],
                [5, 6], [5, 7], [5, 8], [5, 9],
                [6, 7], [6, 8], [6, 9],
                [7, 8], [7, 9],
                [8, 9],
            ],
        },
    },
    {
        "case": "full_c_n10_d_noloops",
        "origin": "mirrors igraph_full(n=10, directed=true, loops=false) — directed K_10, 90 arcs (Directed, no loops)",
        "algo": "full_graph",
        "params": {"n": 10, "directed": True, "loops": False},
        "expected": {
            "vcount": 10,
            "ecount": 90,
            "directed": True,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4], [0, 5], [0, 6], [0, 7], [0, 8], [0, 9],
                [1, 0], [1, 2], [1, 3], [1, 4], [1, 5], [1, 6], [1, 7], [1, 8], [1, 9],
                [2, 0], [2, 1], [2, 3], [2, 4], [2, 5], [2, 6], [2, 7], [2, 8], [2, 9],
                [3, 0], [3, 1], [3, 2], [3, 4], [3, 5], [3, 6], [3, 7], [3, 8], [3, 9],
                [4, 0], [4, 1], [4, 2], [4, 3], [4, 5], [4, 6], [4, 7], [4, 8], [4, 9],
                [5, 0], [5, 1], [5, 2], [5, 3], [5, 4], [5, 6], [5, 7], [5, 8], [5, 9],
                [6, 0], [6, 1], [6, 2], [6, 3], [6, 4], [6, 5], [6, 7], [6, 8], [6, 9],
                [7, 0], [7, 1], [7, 2], [7, 3], [7, 4], [7, 5], [7, 6], [7, 8], [7, 9],
                [8, 0], [8, 1], [8, 2], [8, 3], [8, 4], [8, 5], [8, 6], [8, 7], [8, 9],
                [9, 0], [9, 1], [9, 2], [9, 3], [9, 4], [9, 5], [9, 6], [9, 7], [9, 8],
            ],
        },
    },
    {
        "case": "full_c_n10_ud_loops",
        "origin": "mirrors igraph_full(n=10, directed=false, loops=true) — undirected K_10 + self-loops, 55 edges (Undirected, with loops)",
        "algo": "full_graph",
        "params": {"n": 10, "directed": False, "loops": True},
        "expected": {
            "vcount": 10,
            "ecount": 55,
            "directed": False,
            "edges": [
                [0, 0], [0, 1], [0, 2], [0, 3], [0, 4], [0, 5], [0, 6], [0, 7], [0, 8], [0, 9],
                [1, 1], [1, 2], [1, 3], [1, 4], [1, 5], [1, 6], [1, 7], [1, 8], [1, 9],
                [2, 2], [2, 3], [2, 4], [2, 5], [2, 6], [2, 7], [2, 8], [2, 9],
                [3, 3], [3, 4], [3, 5], [3, 6], [3, 7], [3, 8], [3, 9],
                [4, 4], [4, 5], [4, 6], [4, 7], [4, 8], [4, 9],
                [5, 5], [5, 6], [5, 7], [5, 8], [5, 9],
                [6, 6], [6, 7], [6, 8], [6, 9],
                [7, 7], [7, 8], [7, 9],
                [8, 8], [8, 9],
                [9, 9],
            ],
        },
    },
    {
        "case": "full_c_n10_d_loops",
        "origin": "mirrors igraph_full(n=10, directed=true, loops=true) — directed K_10 + self-loops, 100 arcs (Directed, with loops)",
        "algo": "full_graph",
        "params": {"n": 10, "directed": True, "loops": True},
        "expected": {
            "vcount": 10,
            "ecount": 100,
            "directed": True,
            "edges": [
                [0, 0], [0, 1], [0, 2], [0, 3], [0, 4], [0, 5], [0, 6], [0, 7], [0, 8], [0, 9],
                [1, 0], [1, 1], [1, 2], [1, 3], [1, 4], [1, 5], [1, 6], [1, 7], [1, 8], [1, 9],
                [2, 0], [2, 1], [2, 2], [2, 3], [2, 4], [2, 5], [2, 6], [2, 7], [2, 8], [2, 9],
                [3, 0], [3, 1], [3, 2], [3, 3], [3, 4], [3, 5], [3, 6], [3, 7], [3, 8], [3, 9],
                [4, 0], [4, 1], [4, 2], [4, 3], [4, 4], [4, 5], [4, 6], [4, 7], [4, 8], [4, 9],
                [5, 0], [5, 1], [5, 2], [5, 3], [5, 4], [5, 5], [5, 6], [5, 7], [5, 8], [5, 9],
                [6, 0], [6, 1], [6, 2], [6, 3], [6, 4], [6, 5], [6, 6], [6, 7], [6, 8], [6, 9],
                [7, 0], [7, 1], [7, 2], [7, 3], [7, 4], [7, 5], [7, 6], [7, 7], [7, 8], [7, 9],
                [8, 0], [8, 1], [8, 2], [8, 3], [8, 4], [8, 5], [8, 6], [8, 7], [8, 8], [8, 9],
                [9, 0], [9, 1], [9, 2], [9, 3], [9, 4], [9, 5], [9, 6], [9, 7], [9, 8], [9, 9],
            ],
        },
    },
]


# ALGO-CN-025 — `igraph_full_citation` from `src/constructors/full.c`.
# Upstream unit test (`tests/unit/igraph_full_citation.c`) covers four
# cases: n=4 undirected (K_4), n=4 directed (complete DAG with descending
# arcs), n=1 directed (edgeless singleton), n=0 directed (empty graph).
# The directed branch emits arcs in citation order `(i, j)` for every
# `j < i`; the undirected branch yields the same multiset as a complete
# graph but the emission order is descending-source-major.
FULL_CITATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_citation_c_n4_undirected",
        "origin": "mirrors igraph_full_citation(n=4, directed=false) — K_4 (Undirected case from tests/unit/igraph_full_citation.c)",
        "algo": "full_citation",
        "params": {"n": 4, "directed": False},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [[1, 0], [2, 0], [2, 1], [3, 0], [3, 1], [3, 2]],
        },
    },
    {
        "case": "full_citation_c_n4_directed",
        "origin": "mirrors igraph_full_citation(n=4, directed=true) — complete DAG with arcs i->j for every j<i (Directed case from tests/unit/igraph_full_citation.c)",
        "algo": "full_citation",
        "params": {"n": 4, "directed": True},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": True,
            "edges": [[1, 0], [2, 0], [2, 1], [3, 0], [3, 1], [3, 2]],
        },
    },
    {
        "case": "full_citation_c_n1_directed",
        "origin": "mirrors igraph_full_citation(n=1, directed=true) — edgeless singleton (Directed, 1 vertex from tests/unit/igraph_full_citation.c)",
        "algo": "full_citation",
        "params": {"n": 1, "directed": True},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
    {
        "case": "full_citation_c_n0_directed",
        "origin": "mirrors igraph_full_citation(n=0, directed=true) — empty graph (Directed, 0 vertices from tests/unit/igraph_full_citation.c)",
        "algo": "full_citation",
        "params": {"n": 0, "directed": True},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]


# ALGO-CN-026 — `igraph_full_multipartite` from `src/constructors/full.c`.
# Upstream unit test (`tests/unit/igraph_full_multipartite.c`) covers
# seven cases: (1) empty directed, (2) single partition n=[4] directed,
# (3) three partitions n=[2,3,3] directed ALL (8 vertices, 42 arcs),
# (4) four partitions n=[2,3,4,2] directed IN (11 vertices, 44 arcs),
# (5) four partitions n=[2,3,4,2] undirected (11 vertices, 44 edges),
# (6) all-zero partitions n=[0,0,0] directed, (7) partition with one
# size-zero block n=[2,0,3] directed ALL (5 vertices, 12 arcs).
# Modes use the conventional igraph_neimode_t spelling: "all" / "out" /
# "in". The expected edge multisets are byte-for-byte copies of the
# upstream `igraph_full_multipartite.out`. Comparison is multiset-based
# (no emission-order assumption) so the test passes under any of igraph
# C / python-igraph / R-igraph backends.
FULL_MULTIPARTITE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_multipartite_c_empty_directed_all",
        "origin": "mirrors igraph_full_multipartite(n=[], directed=true, mode=ALL) — empty graph, empty types (case 1 from tests/unit/igraph_full_multipartite.out)",
        "algo": "full_multipartite",
        "params": {"partitions": [], "directed": True, "mode": "all"},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
            "types": [],
        },
    },
    {
        "case": "full_multipartite_c_single_partition_n4_directed_all",
        "origin": "mirrors igraph_full_multipartite(n=[4], directed=true, mode=ALL) — 4 isolated vertices, no edges, types=[0,0,0,0] (case 2 from tests/unit/igraph_full_multipartite.out)",
        "algo": "full_multipartite",
        "params": {"partitions": [4], "directed": True, "mode": "all"},
        "expected": {
            "vcount": 4,
            "ecount": 0,
            "directed": True,
            "edges": [],
            "types": [0, 0, 0, 0],
        },
    },
    {
        "case": "full_multipartite_c_three_partitions_2_3_3_directed_all",
        "origin": "mirrors igraph_full_multipartite(n=[2,3,3], directed=true, mode=ALL) — 8 vertices, 42 mutual arcs across 21 inter-partition pairs (case 3 from tests/unit/igraph_full_multipartite.out)",
        "algo": "full_multipartite",
        "params": {"partitions": [2, 3, 3], "directed": True, "mode": "all"},
        "expected": {
            "vcount": 8,
            "ecount": 42,
            "directed": True,
            "edges": [
                [0, 2], [2, 0], [0, 3], [3, 0], [0, 4], [4, 0],
                [0, 5], [5, 0], [0, 6], [6, 0], [0, 7], [7, 0],
                [1, 2], [2, 1], [1, 3], [3, 1], [1, 4], [4, 1],
                [1, 5], [5, 1], [1, 6], [6, 1], [1, 7], [7, 1],
                [2, 5], [5, 2], [2, 6], [6, 2], [2, 7], [7, 2],
                [3, 5], [5, 3], [3, 6], [6, 3], [3, 7], [7, 3],
                [4, 5], [5, 4], [4, 6], [6, 4], [4, 7], [7, 4],
            ],
            "types": [0, 0, 1, 1, 1, 2, 2, 2],
        },
    },
    {
        "case": "full_multipartite_c_four_partitions_2_3_4_2_directed_in",
        "origin": "mirrors igraph_full_multipartite(n=[2,3,4,2], directed=true, mode=IN) — 11 vertices, 44 reversed arcs (case 4 from tests/unit/igraph_full_multipartite.out)",
        "algo": "full_multipartite",
        "params": {"partitions": [2, 3, 4, 2], "directed": True, "mode": "in"},
        "expected": {
            "vcount": 11,
            "ecount": 44,
            "directed": True,
            "edges": [
                [2, 0], [3, 0], [4, 0], [5, 0], [6, 0], [7, 0], [8, 0], [9, 0], [10, 0],
                [2, 1], [3, 1], [4, 1], [5, 1], [6, 1], [7, 1], [8, 1], [9, 1], [10, 1],
                [5, 2], [6, 2], [7, 2], [8, 2], [9, 2], [10, 2],
                [5, 3], [6, 3], [7, 3], [8, 3], [9, 3], [10, 3],
                [5, 4], [6, 4], [7, 4], [8, 4], [9, 4], [10, 4],
                [9, 5], [10, 5], [9, 6], [10, 6], [9, 7], [10, 7], [9, 8], [10, 8],
            ],
            "types": [0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3],
        },
    },
    {
        "case": "full_multipartite_c_four_partitions_2_3_4_2_undirected_all",
        "origin": "mirrors igraph_full_multipartite(n=[2,3,4,2], directed=false, mode=ALL) — 11 vertices, 44 undirected edges (case 5 from tests/unit/igraph_full_multipartite.out)",
        "algo": "full_multipartite",
        "params": {"partitions": [2, 3, 4, 2], "directed": False, "mode": "all"},
        "expected": {
            "vcount": 11,
            "ecount": 44,
            "directed": False,
            "edges": [
                [0, 2], [0, 3], [0, 4], [0, 5], [0, 6], [0, 7], [0, 8], [0, 9], [0, 10],
                [1, 2], [1, 3], [1, 4], [1, 5], [1, 6], [1, 7], [1, 8], [1, 9], [1, 10],
                [2, 5], [2, 6], [2, 7], [2, 8], [2, 9], [2, 10],
                [3, 5], [3, 6], [3, 7], [3, 8], [3, 9], [3, 10],
                [4, 5], [4, 6], [4, 7], [4, 8], [4, 9], [4, 10],
                [5, 9], [5, 10], [6, 9], [6, 10], [7, 9], [7, 10], [8, 9], [8, 10],
            ],
            "types": [0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3],
        },
    },
    {
        "case": "full_multipartite_c_all_zero_partitions_directed_all",
        "origin": "mirrors igraph_full_multipartite(n=[0,0,0], directed=true, mode=ALL) — empty graph despite three nominal partitions, types=[] (case 6 from tests/unit/igraph_full_multipartite.out)",
        "algo": "full_multipartite",
        "params": {"partitions": [0, 0, 0], "directed": True, "mode": "all"},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
            "types": [],
        },
    },
    {
        "case": "full_multipartite_c_one_empty_partition_2_0_3_directed_all",
        "origin": "mirrors igraph_full_multipartite(n=[2,0,3], directed=true, mode=ALL) — 5 vertices, 12 arcs across K_{2,3} bipartite-with-skipped-partition (case 7 from tests/unit/igraph_full_multipartite.out)",
        "algo": "full_multipartite",
        "params": {"partitions": [2, 0, 3], "directed": True, "mode": "all"},
        "expected": {
            "vcount": 5,
            "ecount": 12,
            "directed": True,
            "edges": [
                [0, 2], [2, 0], [0, 3], [3, 0], [0, 4], [4, 0],
                [1, 2], [2, 1], [1, 3], [3, 1], [1, 4], [4, 1],
            ],
            "types": [0, 0, 2, 2, 2],
        },
    },
]


# ALGO-CN-027 — `igraph_turan` from `src/constructors/full.c:281-325`.
# Upstream unit test (`tests/unit/igraph_turan.c`) emits six cases in
# `igraph_turan.out`: (1) n=0, r=10 → empty + empty types, (2) n=10, r=1
# → 10 isolated vertices, (3) n=4, r=6 → capped to r=4 yielding K_4
# (types = [0,1,2,3]), (4) n=13, r=4 → 63 edges across partitions
# [4,3,3,3] (types = [0,0,0,0,1,1,1,2,2,2,3,3,3]), (5) n=8, r=3 → 21
# edges across [3,3,2] (types = [0,0,0,1,1,1,2,2]), (6) n=6, r=3 → 12
# edges across [2,2,2] (types = [0,0,1,1,2,2], the octahedron / cocktail
# party graph). Edge multisets are byte-for-byte copies from the upstream
# `.out`. The Rust constructor is undirected only, matching upstream.
TURAN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "turan_c_n0_r10_empty",
        "origin": "mirrors igraph_turan(n=0, r=10) — empty graph, empty types (case 1 from tests/unit/igraph_turan.out)",
        "algo": "turan",
        "params": {"n": 0, "r": 10},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": False,
            "edges": [],
            "types": [],
        },
    },
    {
        "case": "turan_c_n10_r1_isolated",
        "origin": "mirrors igraph_turan(n=10, r=1) — 10 isolated vertices, types=[0]*10 (case 2 from tests/unit/igraph_turan.out)",
        "algo": "turan",
        "params": {"n": 10, "r": 1},
        "expected": {
            "vcount": 10,
            "ecount": 0,
            "directed": False,
            "edges": [],
            "types": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        },
    },
    {
        "case": "turan_c_n4_r6_capped_k4",
        "origin": "mirrors igraph_turan(n=4, r=6) — r capped to n=4 yielding K_4, types=[0,1,2,3] (case 3 from tests/unit/igraph_turan.out)",
        "algo": "turan",
        "params": {"n": 4, "r": 6},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]],
            "types": [0, 1, 2, 3],
        },
    },
    {
        "case": "turan_c_n13_r4_partitions_4_3_3_3",
        "origin": "mirrors igraph_turan(n=13, r=4) — partitions [4,3,3,3], 63 edges, isomorphic to full_multipartite(4,3,3,3) (case 4 from tests/unit/igraph_turan.out)",
        "algo": "turan",
        "params": {"n": 13, "r": 4},
        "expected": {
            "vcount": 13,
            "ecount": 63,
            "directed": False,
            "edges": [
                [0, 4], [0, 5], [0, 6], [0, 7], [0, 8], [0, 9], [0, 10], [0, 11], [0, 12],
                [1, 4], [1, 5], [1, 6], [1, 7], [1, 8], [1, 9], [1, 10], [1, 11], [1, 12],
                [2, 4], [2, 5], [2, 6], [2, 7], [2, 8], [2, 9], [2, 10], [2, 11], [2, 12],
                [3, 4], [3, 5], [3, 6], [3, 7], [3, 8], [3, 9], [3, 10], [3, 11], [3, 12],
                [4, 7], [4, 8], [4, 9], [4, 10], [4, 11], [4, 12],
                [5, 7], [5, 8], [5, 9], [5, 10], [5, 11], [5, 12],
                [6, 7], [6, 8], [6, 9], [6, 10], [6, 11], [6, 12],
                [7, 10], [7, 11], [7, 12],
                [8, 10], [8, 11], [8, 12],
                [9, 10], [9, 11], [9, 12],
            ],
            "types": [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3],
        },
    },
    {
        "case": "turan_c_n8_r3_partitions_3_3_2",
        "origin": "mirrors igraph_turan(n=8, r=3) — partitions [3,3,2], 21 edges, isomorphic to full_multipartite(3,3,2) (case 5 from tests/unit/igraph_turan.out)",
        "algo": "turan",
        "params": {"n": 8, "r": 3},
        "expected": {
            "vcount": 8,
            "ecount": 21,
            "directed": False,
            "edges": [
                [0, 3], [0, 4], [0, 5], [0, 6], [0, 7],
                [1, 3], [1, 4], [1, 5], [1, 6], [1, 7],
                [2, 3], [2, 4], [2, 5], [2, 6], [2, 7],
                [3, 6], [3, 7], [4, 6], [4, 7], [5, 6], [5, 7],
            ],
            "types": [0, 0, 0, 1, 1, 1, 2, 2],
        },
    },
    {
        "case": "turan_c_n6_r3_octahedron",
        "origin": "mirrors igraph_turan(n=6, r=3) — partitions [2,2,2], 12 edges, the cocktail-party graph K_{2,2,2} / octahedron, isomorphic to full_multipartite(2,2,2) (case 6 from tests/unit/igraph_turan.out)",
        "algo": "turan",
        "params": {"n": 6, "r": 3},
        "expected": {
            "vcount": 6,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 2], [0, 3], [0, 4], [0, 5],
                [1, 2], [1, 3], [1, 4], [1, 5],
                [2, 4], [2, 5], [3, 4], [3, 5],
            ],
            "types": [0, 0, 1, 1, 2, 2],
        },
    },
]


# ALGO-CN-028 — `igraph_extended_chordal_ring` from
# `src/constructors/regular.c:868-963`. Upstream unit test
# (`tests/unit/igraph_extended_chordal_ring.c`) covers three cases:
#   (1) n=5, W=[[2]], directed — pentagram + 5-cycle with all chords
#       drawn two steps clockwise (10 directed edges total).
#   (1b) n=5, W=[[-3]], directed — equivalent to case 1 by Euclidean wrap:
#        (i − 3) ≡ (i + 2) (mod 5). Same 10-edge digraph.
#   (2) n=12, W=[[4, 2], [8, 10]], undirected — the "from-article" case
#       where igraph deliberately emits double-edges: chord row 0 column 0
#       (offset 4 on even i) and chord row 1 column 0 (offset 8 ≡ -4) both
#       collapse to the same undirected chord; same on the odd side with
#       offsets 2 and 10 ≡ -2. 12 backbone + 12 × 2 chord = 36 edges in
#       the resulting multigraph.
# Edge lists below were computed by the same algorithm igraph C uses and
# the canonical-form (lo, hi) is normalised for undirected fixtures so the
# multiset comparison in the conformance harness lines up regardless of
# in-graph endpoint order.
EXTENDED_CHORDAL_RING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "extended_chordal_ring_c_pentagram_pos",
        "origin": "mirrors igraph_extended_chordal_ring(nodes=5, W=[[2]], directed=true) — 5-cycle plus chord offset +2 (case 1 from tests/unit/igraph_extended_chordal_ring.c)",
        "algo": "extended_chordal_ring",
        "params": {
            "nodes": 5,
            "w": [[2]],
            "directed": True,
        },
        "expected": {
            "vcount": 5,
            "ecount": 10,
            "directed": True,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 0],
                [0, 2], [1, 3], [2, 4], [3, 0], [4, 1],
            ],
        },
    },
    {
        "case": "extended_chordal_ring_c_pentagram_neg_equivalent",
        "origin": "mirrors igraph_extended_chordal_ring(nodes=5, W=[[-3]], directed=true) — equivalent to W=[[2]] because (i − 3) ≡ (i + 2) (mod 5) (case 1b from tests/unit/igraph_extended_chordal_ring.c)",
        "algo": "extended_chordal_ring",
        "params": {
            "nodes": 5,
            "w": [[-3]],
            "directed": True,
        },
        "expected": {
            "vcount": 5,
            "ecount": 10,
            "directed": True,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 0],
                [0, 2], [1, 3], [2, 4], [3, 0], [4, 1],
            ],
        },
    },
    {
        "case": "extended_chordal_ring_c_article_12_multigraph",
        "origin": "mirrors igraph_extended_chordal_ring(nodes=12, W=[[4,2],[8,10]], undirected) — the 'from article' multigraph case where every chord appears twice (case 2 from tests/unit/igraph_extended_chordal_ring.c)",
        "algo": "extended_chordal_ring",
        "params": {
            "nodes": 12,
            "w": [[4, 2], [8, 10]],
            "directed": False,
        },
        "expected": {
            "vcount": 12,
            "ecount": 36,
            "directed": False,
            "edges": [
                # 12 backbone edges, each multiplicity 1.
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6],
                [6, 7], [7, 8], [8, 9], [9, 10], [10, 11], [0, 11],
                # 6 distinct even-side chord edges, each multiplicity 2.
                [0, 4], [0, 4], [2, 6], [2, 6], [4, 8], [4, 8],
                [6, 10], [6, 10], [0, 8], [0, 8], [2, 10], [2, 10],
                # 6 distinct odd-side chord edges, each multiplicity 2.
                [1, 3], [1, 3], [3, 5], [3, 5], [5, 7], [5, 7],
                [7, 9], [7, 9], [9, 11], [9, 11], [1, 11], [1, 11],
            ],
        },
    },
]


# ALGO-CN-029 — `igraph_adjacency` from `src/constructors/adjacency.c:335-386`.
# The upstream unit test (`tests/unit/igraph_adjacency.c`) walks every
# (mode × loop) combination with three carefully chosen matrices:
#   M3      = [[4,2,0],[3,0,4],[0,5,6]]      — asymmetric, used by
#             DIRECTED / MAX / PLUS / UPPER / LOWER (+ MIN+LOOPS_ONCE/TWICE).
#   M3_SYM  = [[4,2,0],[2,0,4],[0,4,6]]      — symmetric, used by UNDIRECTED.
#   M3_MIN  = [[4,2,0],[3,0,5],[0,4,6]]      — used solely by MIN+NO_LOOPS so
#             the MIN result differs from MAX (both pairs share their min).
# Per-mode loop collapse: LOOPS_TWICE behaves as LOOPS_ONCE for DIRECTED,
# UPPER and LOWER (matrix only stores one half-edge per loop in those
# layouts). The fixtures below canonicalise undirected edges to (min, max)
# so the multiset comparison in the conformance harness is endpoint-order
# agnostic.
ADJACENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "adjacency_c_0x0_directed_loops_once",
        "origin": "mirrors igraph_adjacency(0x0 matrix, IGRAPH_ADJ_DIRECTED, IGRAPH_LOOPS_ONCE) — empty directed graph (tests/unit/igraph_adjacency.c, lines 66-68)",
        "algo": "adjacency",
        "params": {
            "matrix": [],
            "mode": "directed",
            "loops": "once",
        },
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
    {
        "case": "adjacency_c_1x1_directed_loops_once",
        "origin": "mirrors igraph_adjacency([[1]], IGRAPH_ADJ_DIRECTED, IGRAPH_LOOPS_ONCE) — single self-loop (tests/unit/igraph_adjacency.c, lines 76-81)",
        "algo": "adjacency",
        "params": {
            "matrix": [[1]],
            "mode": "directed",
            "loops": "once",
        },
        "expected": {
            "vcount": 1,
            "ecount": 1,
            "directed": True,
            "edges": [[0, 0]],
        },
    },
    {
        "case": "adjacency_c_1x1_directed_loops_twice_collapsed",
        "origin": "mirrors igraph_adjacency([[1]], IGRAPH_ADJ_DIRECTED, IGRAPH_LOOPS_TWICE) — DIRECTED collapses TWICE→ONCE so a single loop emits (tests/unit/igraph_adjacency.c, lines 82-87)",
        "algo": "adjacency",
        "params": {
            "matrix": [[1]],
            "mode": "directed",
            "loops": "twice",
        },
        "expected": {
            "vcount": 1,
            "ecount": 1,
            "directed": True,
            "edges": [[0, 0]],
        },
    },
    {
        "case": "adjacency_c_3x3_directed_no_loops",
        "origin": "mirrors igraph_adjacency(M3=[[4,2,0],[3,0,4],[0,5,6]], IGRAPH_ADJ_DIRECTED, IGRAPH_NO_LOOPS) — 14 directed off-diagonal arcs (tests/unit/igraph_adjacency.c, lines 89-94)",
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
        "case": "adjacency_c_3x3_directed_loops_once",
        "origin": "mirrors igraph_adjacency(M3, IGRAPH_ADJ_DIRECTED, IGRAPH_LOOPS_ONCE) — off-diagonal arcs plus diag {4,0,6} as loops (tests/unit/igraph_adjacency.c, lines 95-100)",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [3, 0, 4], [0, 5, 6]],
            "mode": "directed",
            "loops": "once",
        },
        "expected": {
            "vcount": 3,
            "ecount": 24,
            "directed": True,
            "edges": [
                [0, 0], [0, 0], [0, 0], [0, 0],
                [0, 1], [0, 1],
                [1, 0], [1, 0], [1, 0],
                [1, 2], [1, 2], [1, 2], [1, 2],
                [2, 1], [2, 1], [2, 1], [2, 1], [2, 1],
                [2, 2], [2, 2], [2, 2], [2, 2], [2, 2], [2, 2],
            ],
        },
    },
    {
        "case": "adjacency_c_3x3_undirected_loops_twice",
        "origin": "mirrors igraph_adjacency(M3_SYM=[[4,2,0],[2,0,4],[0,4,6]], IGRAPH_ADJ_UNDIRECTED, IGRAPH_LOOPS_TWICE) — diagonal {4,0,6} halved to {2,0,3} loops, off-diag {2,4} carried as-is (tests/unit/igraph_adjacency.c, lines 119-124)",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [2, 0, 4], [0, 4, 6]],
            "mode": "undirected",
            "loops": "twice",
        },
        "expected": {
            "vcount": 3,
            "ecount": 11,
            "directed": False,
            "edges": [
                [0, 0], [0, 0],
                [0, 1], [0, 1],
                [1, 2], [1, 2], [1, 2], [1, 2],
                [2, 2], [2, 2], [2, 2],
            ],
        },
    },
    {
        "case": "adjacency_c_3x3_max_no_loops",
        "origin": "mirrors igraph_adjacency(M3, IGRAPH_ADJ_MAX, IGRAPH_NO_LOOPS) — pair (i,j) gets max(A[i,j], A[j,i]) edges; (0,1)=max(2,3)=3, (1,2)=max(4,5)=5 (tests/unit/igraph_adjacency.c, lines 125-130)",
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
        "case": "adjacency_c_3x3_min_no_loops",
        "origin": "mirrors igraph_adjacency(M3_MIN=[[4,2,0],[3,0,5],[0,4,6]], IGRAPH_ADJ_MIN, IGRAPH_NO_LOOPS) — pair (i,j) gets min(A[i,j], A[j,i]); (0,1)=min(2,3)=2, (1,2)=min(5,4)=4 (tests/unit/igraph_adjacency.c, lines 143-148)",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [3, 0, 5], [0, 4, 6]],
            "mode": "min",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 1], [0, 1],
                [1, 2], [1, 2], [1, 2], [1, 2],
            ],
        },
    },
    {
        "case": "adjacency_c_3x3_plus_no_loops",
        "origin": "mirrors igraph_adjacency(M3, IGRAPH_ADJ_PLUS, IGRAPH_NO_LOOPS) — pair (i,j) gets A[i,j]+A[j,i]; (0,1)=2+3=5, (1,2)=4+5=9 (tests/unit/igraph_adjacency.c, lines 161-166)",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [3, 0, 4], [0, 5, 6]],
            "mode": "plus",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 14,
            "directed": False,
            "edges": [
                [0, 1], [0, 1], [0, 1], [0, 1], [0, 1],
                [1, 2], [1, 2], [1, 2], [1, 2], [1, 2], [1, 2], [1, 2], [1, 2], [1, 2],
            ],
        },
    },
    {
        "case": "adjacency_c_3x3_upper_loops_twice_collapsed",
        "origin": "mirrors igraph_adjacency(M3, IGRAPH_ADJ_UPPER, IGRAPH_LOOPS_TWICE) — UPPER collapses TWICE→ONCE so diag {4,0,6} = 4+6 loops; off-diag (0,1)=2, (1,2)=4 (tests/unit/igraph_adjacency.c, lines 191-196)",
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
    {
        "case": "adjacency_c_3x3_lower_no_loops",
        "origin": "mirrors igraph_adjacency(M3, IGRAPH_ADJ_LOWER, IGRAPH_NO_LOOPS) — lower-triangle entries M[1,0]=3 and M[2,1]=5 produce 3 and 5 undirected edges (tests/unit/igraph_adjacency.c, lines 197-202)",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [3, 0, 4], [0, 5, 6]],
            "mode": "lower",
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
]


# ALGO-CN-030 — `igraph_weighted_adjacency` from `src/constructors/adjacency.c`.
# Real-valued sibling of `igraph_adjacency`. Same 7-mode × 3-loops dispatch.
# Each non-zero cell produces exactly ONE edge whose weight is the cell's
# value (after the per-mode reduction). Loop-weight adjustment per mode:
# NoLoops drops the diagonal, Twice halves it, Once passes through.
# DIRECTED / UPPER / LOWER all collapse Twice → Once.
# Undirected accepts pairs where both sides are NaN as "symmetric".
# These fixtures mirror the upstream C unit test
# (tests/unit/igraph_weighted_adjacency.c) and the values are hand-checked.
# Edges are canonicalised to (min, max) for undirected variants so the
# conformance harness can compare as an order-agnostic multiset.
WEIGHTED_ADJACENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "weighted_adjacency_c_0x0_directed_loops_once",
        "origin": "mirrors igraph_weighted_adjacency(0x0, IGRAPH_ADJ_DIRECTED, IGRAPH_LOOPS_ONCE) — empty directed graph (tests/unit/igraph_weighted_adjacency.c)",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [],
            "mode": "directed",
            "loops": "once",
        },
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
            "weights": [],
        },
    },
    {
        "case": "weighted_adjacency_c_1x1_directed_loops_once",
        "origin": "mirrors igraph_weighted_adjacency([[1.5]], IGRAPH_ADJ_DIRECTED, IGRAPH_LOOPS_ONCE) — single weighted self-loop",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[1.5]],
            "mode": "directed",
            "loops": "once",
        },
        "expected": {
            "vcount": 1,
            "ecount": 1,
            "directed": True,
            "edges": [[0, 0]],
            "weights": [1.5],
        },
    },
    {
        "case": "weighted_adjacency_c_1x1_directed_loops_twice_collapsed",
        "origin": "mirrors igraph_weighted_adjacency([[1.5]], IGRAPH_ADJ_DIRECTED, IGRAPH_LOOPS_TWICE) — DIRECTED collapses TWICE→ONCE so diag weight passes through unhalved",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[1.5]],
            "mode": "directed",
            "loops": "twice",
        },
        "expected": {
            "vcount": 1,
            "ecount": 1,
            "directed": True,
            "edges": [[0, 0]],
            "weights": [1.5],
        },
    },
    {
        "case": "weighted_adjacency_c_3x3_directed_no_loops",
        "origin": "mirrors igraph_weighted_adjacency(M3=[[2.0,0.5,0],[1.5,0,2.0],[0,2.5,3.0]], IGRAPH_ADJ_DIRECTED, IGRAPH_NO_LOOPS) — column-major emit drops diagonal, 4 off-diagonal non-zeros",
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
            # column-major: j=0 i=1 (1,0)=1.5; j=1 i=0 (0,1)=0.5, i=2 (2,1)=2.5; j=2 i=1 (1,2)=2.0
            "edges": [[1, 0], [0, 1], [2, 1], [1, 2]],
            "weights": [1.5, 0.5, 2.5, 2.0],
        },
    },
    {
        "case": "weighted_adjacency_c_3x3_directed_loops_once",
        "origin": "mirrors igraph_weighted_adjacency(M3, IGRAPH_ADJ_DIRECTED, IGRAPH_LOOPS_ONCE) — diagonal non-zeros (2.0, 3.0) emit as un-halved self-loops",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [1.5, 0.0, 2.0], [0.0, 2.5, 3.0]],
            "mode": "directed",
            "loops": "once",
        },
        "expected": {
            "vcount": 3,
            "ecount": 6,
            "directed": True,
            # column-major: j=0 (0,0)=2.0, (1,0)=1.5; j=1 (0,1)=0.5, (2,1)=2.5; j=2 (1,2)=2.0, (2,2)=3.0
            "edges": [[0, 0], [1, 0], [0, 1], [2, 1], [1, 2], [2, 2]],
            "weights": [2.0, 1.5, 0.5, 2.5, 2.0, 3.0],
        },
    },
    {
        "case": "weighted_adjacency_c_3x3_undirected_loops_twice",
        "origin": "mirrors igraph_weighted_adjacency(M3_SYM=[[2.0,0.5,0],[0.5,0,2.0],[0,2.0,3.0]], IGRAPH_ADJ_UNDIRECTED, IGRAPH_LOOPS_TWICE) — diagonal weights halved (2.0→1.0, 3.0→1.5), off-diagonal pass-through",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [0.5, 0.0, 2.0], [0.0, 2.0, 3.0]],
            "mode": "undirected",
            "loops": "twice",
        },
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": False,
            # row-major lower walk: i=0 diag 2.0→1.0; i=1 (1,0)=0.5; i=2 diag 3.0→1.5, (2,1)=2.0
            "edges": [[0, 0], [0, 1], [2, 2], [1, 2]],
            "weights": [1.0, 0.5, 1.5, 2.0],
        },
    },
    {
        "case": "weighted_adjacency_c_3x3_max_no_loops",
        "origin": "mirrors igraph_weighted_adjacency(M3, IGRAPH_ADJ_MAX, IGRAPH_NO_LOOPS) — pair (i,j) gets max(A[i,j], A[j,i]); (0,1)=max(0.5,1.5)=1.5, (1,2)=max(2.0,2.5)=2.5, (0,2)=0 skipped",
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
        "case": "weighted_adjacency_c_3x3_min_no_loops",
        "origin": "mirrors igraph_weighted_adjacency(M3, IGRAPH_ADJ_MIN, IGRAPH_NO_LOOPS) — pair (i,j) gets min(A[i,j], A[j,i]); (0,1)=min(0.5,1.5)=0.5, (1,2)=min(2.0,2.5)=2.0",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [1.5, 0.0, 2.0], [0.0, 2.5, 3.0]],
            "mode": "min",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 1], [1, 2]],
            "weights": [0.5, 2.0],
        },
    },
    {
        "case": "weighted_adjacency_c_3x3_plus_no_loops",
        "origin": "mirrors igraph_weighted_adjacency(M3, IGRAPH_ADJ_PLUS, IGRAPH_NO_LOOPS) — pair (i,j) gets A[i,j]+A[j,i]; (0,1)=0.5+1.5=2.0, (1,2)=2.0+2.5=4.5",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [1.5, 0.0, 2.0], [0.0, 2.5, 3.0]],
            "mode": "plus",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 1], [1, 2]],
            "weights": [2.0, 4.5],
        },
    },
    {
        "case": "weighted_adjacency_c_3x3_upper_loops_twice_collapsed",
        "origin": "mirrors igraph_weighted_adjacency(M3, IGRAPH_ADJ_UPPER, IGRAPH_LOOPS_TWICE) — UPPER collapses TWICE→ONCE so diag weights pass through unhalved (2.0, 3.0); upper triangle (0,1)=0.5, (1,2)=2.0",
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
            # column-major upper: j=0 diag 2.0; j=1 (0,1)=0.5; j=2 (1,2)=2.0, diag 3.0
            "edges": [[0, 0], [0, 1], [1, 2], [2, 2]],
            "weights": [2.0, 0.5, 2.0, 3.0],
        },
    },
    {
        "case": "weighted_adjacency_c_3x3_lower_no_loops",
        "origin": "mirrors igraph_weighted_adjacency(M3, IGRAPH_ADJ_LOWER, IGRAPH_NO_LOOPS) — strict lower triangle entries M[1,0]=1.5, M[2,1]=2.5; M[2,0]=0 skipped",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [1.5, 0.0, 2.0], [0.0, 2.5, 3.0]],
            "mode": "lower",
            "loops": "no_loops",
        },
        "expected": {
            "vcount": 3,
            "ecount": 2,
            "directed": False,
            # column-major lower (canonicalised to (min, max) for undirected)
            "edges": [[0, 1], [1, 2]],
            "weights": [1.5, 2.5],
        },
    },
]


# ALGO-CN-015 — `igraph_linegraph` from `src/constructors/linegraph.c`.
# The upstream unit test (`tests/unit/igraph_linegraph.c`) covers three
# canonical shapes: (a) a multigraph + self-loop undirected case, (b) a
# multigraph + self-loop directed case, and (c) an empty directed graph.
# We add a couple of small textbook fixtures (P_4 → P_3, K_3 → K_3,
# star S_4) for quick smoke coverage. Expected edge lists were computed
# via `python-igraph`'s `Graph.linegraph()` (which dispatches to the
# same C entry point), then canonicalised to (min, max) for undirected
# variants to match the way our `Graph` stores them.
LINEGRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "linegraph_c_undirected_canonical",
        "origin": "mirrors igraph_linegraph.c: undirected 7-vertex multigraph with a self-loop at 2 (edges 0-1,1-2,1-3,1-3,2-2,2-4,3-4,4-5) — 18 L-edges on 8 L-vertices",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            7,
            edges=[(0, 1), (1, 2), (1, 3), (1, 3), (2, 2), (2, 4), (3, 4), (4, 5)],
            directed=False,
        ),
        "params": {},
        "expected": {
            "vcount": 8,
            "ecount": 18,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [1, 2], [2, 3], [0, 3], [1, 3], [2, 3],
                [1, 4], [1, 4], [4, 4], [1, 5], [4, 5], [4, 5],
                [5, 6], [3, 6], [2, 6], [5, 7], [6, 7],
            ],
        },
    },
    {
        "case": "linegraph_c_directed_canonical",
        "origin": "mirrors igraph_linegraph.c: directed 7-vertex 8-arc graph with a self-loop at 2 (arcs 0-1,1-2,1-3,3-1,2-2,2-4,3-4,4-5) — 12 L-arcs on 8 L-vertices",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            7,
            edges=[(0, 1), (1, 2), (1, 3), (3, 1), (2, 2), (2, 4), (3, 4), (4, 5)],
            directed=True,
        ),
        "params": {},
        "expected": {
            "vcount": 8,
            "ecount": 12,
            "directed": True,
            "edges": [
                [0, 1], [3, 1], [0, 2], [3, 2], [2, 3], [1, 4],
                [4, 4], [1, 5], [4, 5], [2, 6], [5, 7], [6, 7],
            ],
        },
    },
    {
        "case": "linegraph_c_no_edges_directed",
        "origin": "mirrors igraph_linegraph.c: empty directed graph on 7 vertices, 0 arcs — L(G) is the empty 0-vertex directed graph",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(7, edges=[], directed=True),
        "params": {},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
    {
        "case": "linegraph_c_path_p4",
        "origin": "textbook smoke: L(P_4) = P_3 on the three L-vertices (one edge per shared interior vertex)",
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
        "case": "linegraph_c_triangle_k3",
        "origin": "textbook smoke: L(K_3) = K_3 (every pair of edges shares an endpoint)",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            3, edges=[(0, 1), (0, 2), (1, 2)], directed=False
        ),
        "params": {},
        "expected": {
            "vcount": 3,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [1, 2], [0, 2]],
        },
    },
]


# ALGO-CN-016 — `igraph_from_prufer` from `src/constructors/prufer.c`.
# The upstream unit test (`tests/unit/igraph_from_prufer.c` + .out) covers
# three canonical sequences: a 4-element sequence on n=6, a 6-element
# sequence on n=8, and the empty sequence on n=2. Expected edges below
# are taken directly from the `.out` golden, canonicalised to (min, max)
# pairs — the conformance check compares multisets so cross-source edge
# orderings stay compatible.
# ALGO-CN-017 — `igraph_tree_from_parent_vector` from
# `src/constructors/trees.c`. Upstream unit test
# `tests/unit/igraph_tree_from_parent_vector.c` + `.out` covers the same
# five-vertex parent vector `[4, 4, 1, -2, 3]` in OUT / IN / Undirected
# modes plus a two-root forest variant and an all-roots edgeless case.
# Edges below are taken verbatim from the `.out` golden, canonicalised to
# (min, max) pairs in the undirected case so conformance multiset checks
# stay source-agnostic. Mode strings match Rust's TreeMode variants.
TREE_FROM_PARENT_VECTOR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "tree_from_parent_vector_c_out_5v",
        "origin": "mirrors igraph_tree_from_parent_vector.c fixture: parents=[4,4,1,-2,3], OUT mode → directed 5-vertex tree (edges 4→0, 3→4, 4→1, 1→2)",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [4, 4, 1, -2, 3], "mode": "out"},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": True,
            "edges": [[4, 0], [3, 4], [4, 1], [1, 2]],
        },
    },
    {
        "case": "tree_from_parent_vector_c_in_5v",
        "origin": "mirrors igraph_tree_from_parent_vector.c fixture: parents=[4,4,1,-2,3], IN mode → directed 5-vertex tree (edges 0→4, 4→3, 1→4, 2→1)",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [4, 4, 1, -2, 3], "mode": "in"},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 4], [4, 3], [1, 4], [2, 1]],
        },
    },
    {
        "case": "tree_from_parent_vector_c_undirected_5v",
        "origin": "mirrors igraph_tree_from_parent_vector.c fixture: parents=[4,4,1,-2,3], UNDIRECTED mode → 5-vertex tree, canonical (min,max) edges {(0,4),(3,4),(1,4),(1,2)}",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [4, 4, 1, -2, 3], "mode": "undirected"},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 4], [3, 4], [1, 4], [1, 2]],
        },
    },
    {
        "case": "tree_from_parent_vector_c_forest_two_roots",
        "origin": "mirrors igraph_tree_from_parent_vector.c forest fixture: parents=[-1,4,1,-2,3], OUT mode → directed 2-tree forest (edges 4→1, 3→4, 1→2)",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-1, 4, 1, -2, 3], "mode": "out"},
        "expected": {
            "vcount": 5,
            "ecount": 3,
            "directed": True,
            "edges": [[4, 1], [3, 4], [1, 2]],
        },
    },
    {
        "case": "tree_from_parent_vector_c_edgeless_all_roots",
        "origin": "mirrors igraph_tree_from_parent_vector.c edgeless fixture: all-negative parents=[-1,-1,-1,-1,-1] → 5-vertex edgeless graph",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-1, -1, -1, -1, -1], "mode": "out"},
        "expected": {
            "vcount": 5,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
    {
        "case": "tree_from_parent_vector_c_null_graph",
        "origin": "mirrors igraph_tree_from_parent_vector.c null-graph fixture: empty parents → 0-vertex graph",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [], "mode": "out"},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]


# ALGO-CN-018 — `igraph_lcf` from `src/constructors/lcf.c`. The upstream
# unit test (`tests/unit/igraph_lcf.c` + `.out`) asserts ecount/vcount for
# a handful of LCF descriptions and validates Franklin via isomorphism
# with `igraph_famous("franklin")`. We mirror the structural checks and
# pin the resulting canonical edge list — the constructor is deterministic
# so cross-source comparison is unambiguous.
LCF_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lcf_c_franklin_5_minus5_repeats_6",
        "origin": "mirrors igraph_lcf.c Franklin fixture: lcf_small(12, 5, -5, 6, 0) — 12 vertices, 18 edges, isomorphic to igraph_famous(\"franklin\")",
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
        "case": "lcf_c_three_minus2_repeats_4_n8",
        "origin": "mirrors igraph_lcf.c ad-hoc fixture: lcf_small(8, 3, -2, 4, 0) — 8 vertices, 16 edges (Hamilton C_8 + 8 distinct chords)",
        "algo": "lcf",
        "params": {"n": 8, "shifts": [3, -2], "repeats": 4},
        "expected": {
            "vcount": 8,
            "ecount": 16,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7], [0, 7],
                [0, 3], [1, 7], [2, 5], [1, 3], [4, 7], [3, 5], [1, 6], [5, 7],
            ],
        },
    },
    {
        "case": "lcf_c_two_minus2_repeats_2_n2",
        "origin": "mirrors igraph_lcf.c collapse fixture: lcf_small(2, 2, -2, 2, 0) — n=2 forces every chord into a self-loop; simplify collapses to 1 backbone edge",
        "algo": "lcf",
        "params": {"n": 2, "shifts": [2, -2], "repeats": 2},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
    {
        "case": "lcf_c_two_repeats_2_n2",
        "origin": "mirrors igraph_lcf.c collapse fixture: lcf_small(2, 2, 2, 0) — single-shift variant of the above, same 1-edge outcome",
        "algo": "lcf",
        "params": {"n": 2, "shifts": [2], "repeats": 2},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
    {
        "case": "lcf_c_null_graph_bug_996",
        "origin": "mirrors igraph_lcf.c bug #996 regression: lcf_small(0, 0) → 0-vertex empty graph (no shifts, no chord pass)",
        "algo": "lcf",
        "params": {"n": 0, "shifts": [], "repeats": 0},
        "expected": {
            "vcount": 0,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "lcf_c_heawood_5_minus5_repeats_7",
        "origin": "synthetic-but-canonical Heawood fixture: lcf(14, [5,-5], 7) is the LCF description of igraph_famous(\"heawood\") — 14 vertices, 21 edges, bipartite cubic, girth 6",
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
]


# `igraph_mycielskian` / `igraph_mycielski_graph` have no dedicated unit
# test in `references/igraph/tests/unit/` (the algorithm landed without a
# self-test). The fixtures below are synthesised from the published
# Mycielski recurrence `(v', e') = (2v + 1, 3e + v)` and from the
# canonical small cases (M_3 = C_5, M_4 = Grötzsch graph). They are
# C-equivalent in lineage because the rigraph snapshot (`r_*` fixtures
# below) executes the same `igraph_mycielski_graph` C function and lands
# on the same edge multisets.
MYCIELSKI_GRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mycielski_graph_c_k0_null",
        "origin": "mycielski_graph(0) → null graph (k=0 base case)",
        "algo": "mycielski_graph",
        "params": {"k": 0},
        "expected": {"vcount": 0, "ecount": 0, "directed": False, "edges": []},
    },
    {
        "case": "mycielski_graph_c_k1_singleton",
        "origin": "mycielski_graph(1) → singleton (k=1 base case)",
        "algo": "mycielski_graph",
        "params": {"k": 1},
        "expected": {"vcount": 1, "ecount": 0, "directed": False, "edges": []},
    },
    {
        "case": "mycielski_graph_c_k2_p2",
        "origin": "mycielski_graph(2) → P_2 (k=2 base case, single edge)",
        "algo": "mycielski_graph",
        "params": {"k": 2},
        "expected": {"vcount": 2, "ecount": 1, "directed": False, "edges": [[0, 1]]},
    },
    {
        "case": "mycielski_graph_c_k3_c5",
        "origin": "mycielski_graph(3) → C_5 (5-cycle); first non-trivial Mycielski case",
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
        "case": "mycielski_graph_c_k4_grotzsch",
        "origin": "mycielski_graph(4) → Grötzsch graph (11v/20e, triangle-free, χ=4)",
        "algo": "mycielski_graph",
        "params": {"k": 4},
        "expected": {
            "vcount": 11,
            "ecount": 20,
            "directed": False,
            "edges": [
                [0, 1], [0, 3], [1, 2], [2, 4], [3, 4],
                [0, 6], [1, 5], [0, 8], [3, 5], [1, 7],
                [2, 6], [2, 9], [4, 7], [3, 9], [4, 8],
                [5, 10], [6, 10], [7, 10], [8, 10], [9, 10],
            ],
        },
    },
    {
        "case": "mycielski_graph_c_k5_recurrence",
        "origin": "mycielski_graph(5) → 23v/71e (Mycielski recurrence applied once more to Grötzsch)",
        "algo": "mycielski_graph",
        "params": {"k": 5},
        "expected": {
            "vcount": 23,
            "ecount": 71,
            "directed": False,
            # Edge list omitted; the structural recurrence check (vcount + ecount)
            # is enough at this scale and matches what the upstream C API exposes.
            "edges": None,
        },
    },
]


# Fixtures for `igraph_famous`. Each entry pairs a canonical name with
# the exact (vcount, ecount, edges) triple stored in the C source table
# at `references/igraph/src/constructors/famous.c:26-249`. We sample the
# smallest, the canonical mid-size, the largest, plus a couple of aliases
# (case and synonym) so both dispatch paths exercise the same data.
FAMOUS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "famous_c_bull",
        "origin": "igraph_i_famous_bull (famous.c:26) — smallest entry, 5v/5e",
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
        "case": "famous_c_petersen",
        "origin": "igraph_i_famous_petersen (famous.c:180) — canonical 3-regular 10v/15e",
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
        "case": "famous_c_tetrahedron_alias",
        "origin": "igraph_i_famous_tetrahedron via 'Tetrahedral' alias (famous.c:199 + dispatch)",
        "algo": "famous",
        "params": {"name": "Tetrahedral"},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [[0, 3], [1, 3], [2, 3], [0, 1], [1, 2], [0, 2]],
        },
    },
    {
        "case": "famous_c_dodecahedron_case",
        "origin": "igraph_i_famous_dodecahedron via lowercase 'dodecahedron' (case-insensitive dispatch)",
        "algo": "famous",
        "params": {"name": "dodecahedron"},
        "expected": {
            "vcount": 20,
            "ecount": 30,
            "directed": False,
            "edges": [
                [0, 1], [0, 4], [0, 5], [1, 2], [1, 6], [2, 3], [2, 7],
                [3, 4], [3, 8], [4, 9], [5, 10], [5, 11], [6, 10], [6, 14],
                [7, 13], [7, 14], [8, 12], [8, 13], [9, 11], [9, 12],
                [10, 15], [11, 16], [12, 17], [13, 18], [14, 19],
                [15, 16], [15, 19], [16, 17], [17, 18], [18, 19],
            ],
        },
    },
    {
        "case": "famous_c_grotzsch_alias",
        "origin": "igraph_i_famous_grotzsch via 'Groetzsch' German alias (famous.c:82 + dispatch)",
        "algo": "famous",
        "params": {"name": "Groetzsch"},
        "expected": {
            "vcount": 11,
            "ecount": 20,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 7], [0, 10], [1, 3], [1, 6], [1, 9],
                [2, 4], [2, 6], [2, 8], [3, 4], [3, 8], [3, 10],
                [4, 7], [4, 9], [5, 6], [5, 7], [5, 8], [5, 9], [5, 10],
            ],
        },
    },
    {
        "case": "famous_c_zachary_counts",
        "origin": "igraph_i_famous_zachary (famous.c:237) — large 34v/78e karate-club; counts checked, edge list omitted to keep fixture compact",
        "algo": "famous",
        "params": {"name": "Zachary"},
        "expected": {
            "vcount": 34,
            "ecount": 78,
            "directed": False,
            "edges": None,
        },
    },
    {
        "case": "famous_c_meredith_counts",
        "origin": "igraph_i_famous_meredith (famous.c:139) — largest 70v/140e entry; structural-only check",
        "algo": "famous",
        "params": {"name": "Meredith"},
        "expected": {
            "vcount": 70,
            "ecount": 140,
            "directed": False,
            "edges": None,
        },
    },
]


# Fixtures for `igraph_atlas` (ALGO-CN-021). The atlas catalogues every
# simple undirected unlabelled graph on 0..7 vertices in the Read-Wilson
# (1998) ordering (vcount asc, then ecount asc, then degree-sequence lex,
# then automorphism count). Indices selected to sweep across cells:
#  - 0..3 cover the 0-, 1-, 2-vertex cells
#  - 7 / 18 are the K_3 / K_4 last-entry-of-cell graphs
#  - 53 / 208 / 209 / 1252 are cell boundaries (first 6v null, last 6v K_6,
#    first 7v null, last 7v K_7)
#  - 70 and 180 are skipped in python-igraph's connectivity tests
#    (test_atlas.py:174) — included here because the constructor itself
#    handles them fine and we want to catch any regression that changes
#    that.
# Fixtures for `igraph_create` (ALGO-CN-022). The foundational
# edge-list constructor; cases below cover the upstream example file
# (`examples/simple/igraph_create.c`) plus the two error/edge paths the
# C unit test (`tests/unit/igraph_create.c`) probes — both eliminated at
# the Rust type level (no odd-length / no negative IDs) but the shape
# tests still apply.
CREATE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "create_c_upstream_n_zero_infers_four",
        "origin": "examples/simple/igraph_create.c — [0,1, 1,2, 2,3, 2,2] n=0 directed=0",
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
        "case": "create_c_upstream_n_ten_keeps_ten",
        "origin": "examples/simple/igraph_create.c — same edges with n=10 keeps 10 vertices",
        "algo": "create",
        "params": {
            "edges": [[0, 1], [1, 2], [2, 3], [2, 2]],
            "n": 10,
            "directed": False,
        },
        "expected": {
            "vcount": 10,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [2, 2]],
        },
    },
    {
        "case": "create_c_n_smaller_than_max_extends",
        "origin": "basic_constructors.c:53 — n=3 < max+1=7 triggers silent igraph_add_vertices",
        "algo": "create",
        "params": {
            "edges": [[0, 1], [5, 6]],
            "n": 3,
            "directed": False,
        },
        "expected": {
            "vcount": 7,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 1], [5, 6]],
        },
    },
    {
        "case": "create_c_directed_arc_order",
        "origin": "basic_constructors.c:53 directed arc-order preserved",
        "algo": "create",
        "params": {
            "edges": [[0, 1], [1, 0], [1, 2], [2, 1]],
            "n": 3,
            "directed": True,
        },
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 1], [1, 0], [1, 2], [2, 1]],
        },
    },
    {
        "case": "create_c_empty_null_graph",
        "origin": "basic_constructors.c:53 — empty edges + n=0 → null graph",
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
        "case": "create_c_empty_n_positive_isolated",
        "origin": "basic_constructors.c:53 — empty edges + n>0 → isolated vertices",
        "algo": "create",
        "params": {
            "edges": [],
            "n": 5,
            "directed": True,
        },
        "expected": {
            "vcount": 5,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
    {
        "case": "create_c_self_loops_and_parallel",
        "origin": "basic_constructors.c:53 — self-loops + parallels survive (no canonicalisation)",
        "algo": "create",
        "params": {
            "edges": [[0, 0], [1, 1], [0, 1], [0, 1]],
            "n": 0,
            "directed": False,
        },
        "expected": {
            "vcount": 2,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 0], [1, 1], [0, 1], [0, 1]],
        },
    },
]


# Fixtures for `igraph_triangular_lattice` (ALGO-CN-023). The C unit test
# (`tests/unit/igraph_triangular_lattice.c`) walks the four shape branches
# (`dims=[1]`, `dims=[5]`, `dims=[4,5]`, `dims=[3,4,5]`) and the negative-dim
# error path; the negative path is statically unreachable in the Rust port
# because `dims: &[u32]` cannot carry a negative element, so we only mirror
# the structural cases.
TRIANGULAR_LATTICE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "triangular_lattice_c_triangle_side_1_single_vertex",
        "origin": "tests/unit/igraph_triangular_lattice.c — dims=[1] directed=true → singleton",
        "algo": "triangular_lattice",
        "params": {"dims": [1], "directed": True, "mutual": False},
        "expected": {"vcount": 1, "ecount": 0, "directed": True, "edges": []},
    },
    {
        "case": "triangular_lattice_c_triangle_side_3_undirected",
        "origin": "lattices.c:290 — triangle_shape(3), 6 vertices, 9 undirected edges",
        "algo": "triangular_lattice",
        "params": {"dims": [3], "directed": False, "mutual": False},
        "expected": {
            "vcount": 6,
            "ecount": 9,
            "directed": False,
            "edges": [
                [0, 1], [0, 3],
                [1, 2], [1, 3], [1, 4],
                [2, 4],
                [3, 4], [3, 5],
                [4, 5],
            ],
        },
    },
    {
        "case": "triangular_lattice_c_triangle_side_5_directed",
        "origin": "tests/unit/igraph_triangular_lattice.out — Triangular block, 15 v, 30 arcs",
        "algo": "triangular_lattice",
        "params": {"dims": [5], "directed": True, "mutual": False},
        "expected": {
            "vcount": 15,
            "ecount": 30,
            "directed": True,
            "edges": [
                [0, 1], [0, 5],
                [1, 2], [1, 5], [1, 6],
                [2, 3], [2, 6], [2, 7],
                [3, 4], [3, 7], [3, 8],
                [4, 8],
                [5, 6], [5, 9],
                [6, 7], [6, 9], [6, 10],
                [7, 8], [7, 10], [7, 11],
                [8, 11],
                [9, 10], [9, 12],
                [10, 11], [10, 12], [10, 13],
                [11, 13],
                [12, 13], [12, 14],
                [13, 14],
            ],
        },
    },
    {
        "case": "triangular_lattice_c_rectangle_2x2_undirected",
        "origin": "lattices.c:290 — rectangle_shape(2,2), matches python-igraph expectation",
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
        "case": "triangular_lattice_c_rectangle_2x2_directed_mutual",
        "origin": "lattices.c:290 — rectangle 2x2 directed+mutual doubles every undirected edge",
        "algo": "triangular_lattice",
        "params": {"dims": [2, 2], "directed": True, "mutual": True},
        "expected": {
            "vcount": 4,
            "ecount": 10,
            "directed": True,
            "edges": [
                [0, 1], [1, 0],
                [0, 3], [3, 0],
                [0, 2], [2, 0],
                [1, 3], [3, 1],
                [2, 3], [3, 2],
            ],
        },
    },
    {
        "case": "triangular_lattice_c_hexagon_2_2_2_undirected",
        "origin": "lattices.c:290 — hex_shape(2,2,2), 7 vertices, 12 undirected edges",
        "algo": "triangular_lattice",
        "params": {"dims": [2, 2, 2], "directed": False, "mutual": False},
        "expected": {
            "vcount": 7,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 3], [1, 4],
                [2, 3], [2, 5],
                [3, 4], [3, 5], [3, 6],
                [4, 6],
                [5, 6],
            ],
        },
    },
    {
        "case": "triangular_lattice_c_empty_dim_zero",
        "origin": "lattices.c:298 — any dim == 0 collapses to igraph_empty(0, directed)",
        "algo": "triangular_lattice",
        "params": {"dims": [3, 0], "directed": False, "mutual": False},
        "expected": {"vcount": 0, "ecount": 0, "directed": False, "edges": []},
    },
]


def _hex_lattice_expected(dims: List[int], directed: bool, mutual: bool) -> Dict[str, Any]:
    """Compute the canonical (vcount, ecount, edges) payload via the same
    `igraph_hexagonal_lattice` implementation python-igraph wraps —
    keeps the C-extracted manifest faithful without manually copying
    100+ edge lines from the .out file."""
    g = ig.Graph.Hexagonal_Lattice(dims, directed=directed, mutual=mutual)
    return {
        "vcount": g.vcount(),
        "ecount": g.ecount(),
        "directed": bool(g.is_directed()),
        "edges": [list(e.tuple) for e in g.es],
    }


# Fixtures for `igraph_hexagonal_lattice` (ALGO-CN-024). Mirrors
# `tests/unit/igraph_hexagonal_lattice.c` which walks the four shape
# branches (`dims=[1]`, `dims=[5]`, `dims=[4,5]`, `dims=[3,4,5]`) plus
# the empty-graph and 4-dim error paths; the negative-dim path is
# eliminated at the type level by Rust's `&[u32]` signature.
HEXAGONAL_LATTICE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hexagonal_lattice_c_single_hexagon_directed",
        "origin": (
            "tests/unit/igraph_hexagonal_lattice.out — "
            "'Triangular hexagonal lattice, single hexagon' block, dims=[1] dir=true → C_6 (6v, 6 arcs)"
        ),
        "algo": "hexagonal_lattice",
        "params": {"dims": [1], "directed": True, "mutual": False},
        "expected": _hex_lattice_expected([1], True, False),
    },
    {
        "case": "hexagonal_lattice_c_triangle_side_5_directed",
        "origin": (
            "tests/unit/igraph_hexagonal_lattice.out — "
            "'Triangular hexagonal lattice' block, dims=[5] dir=true (46v, 60 arcs)"
        ),
        "algo": "hexagonal_lattice",
        "params": {"dims": [5], "directed": True, "mutual": False},
        "expected": _hex_lattice_expected([5], True, False),
    },
    {
        "case": "hexagonal_lattice_c_rectangle_4x5_directed_mutual",
        "origin": (
            "tests/unit/igraph_hexagonal_lattice.out — "
            "'Rectangular hexagonal lattice' block, dims=[4,5] dir+mut (58v, 154 arcs)"
        ),
        "algo": "hexagonal_lattice",
        "params": {"dims": [4, 5], "directed": True, "mutual": True},
        "expected": _hex_lattice_expected([4, 5], True, True),
    },
    {
        "case": "hexagonal_lattice_c_hexagon_3_4_5_undirected_mutual",
        "origin": (
            "tests/unit/igraph_hexagonal_lattice.out — "
            "'Hexagonal hexagonal lattice' block, dims=[3,4,5] undirected+mutual "
            "(directed=false silently collapses mutual; 94v, 129 edges)"
        ),
        "algo": "hexagonal_lattice",
        "params": {"dims": [3, 4, 5], "directed": False, "mutual": True},
        "expected": _hex_lattice_expected([3, 4, 5], False, True),
    },
    {
        "case": "hexagonal_lattice_c_empty_dim_zero",
        "origin": "lattices.c:580 — any dim == 0 collapses to igraph_empty(0, directed)",
        "algo": "hexagonal_lattice",
        "params": {"dims": [3, 0], "directed": False, "mutual": False},
        "expected": {"vcount": 0, "ecount": 0, "directed": False, "edges": []},
    },
    {
        "case": "hexagonal_lattice_c_empty_dim_zero_directed_keeps_flag",
        "origin": "lattices.c:580 — empty graph still carries `directed` from the call site",
        "algo": "hexagonal_lattice",
        "params": {"dims": [0, 3, 4], "directed": True, "mutual": True},
        "expected": {"vcount": 0, "ecount": 0, "directed": True, "edges": []},
    },
    {
        "case": "hexagonal_lattice_c_rectangle_2x2_undirected",
        "origin": "lattices.c:570 — quasi-rectangle dims=[2,2] (16v, 19 undirected edges)",
        "algo": "hexagonal_lattice",
        "params": {"dims": [2, 2], "directed": False, "mutual": False},
        "expected": _hex_lattice_expected([2, 2], False, False),
    },
]


ATLAS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "atlas_c_null0",
        "origin": "igraph_atlas(0) — null graph on 0 vertices (atlas.c:63-87)",
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
        "case": "atlas_c_k2_single_edge",
        "origin": "igraph_atlas(3) — first 2-vertex non-trivial graph, K_2",
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
        "case": "atlas_c_triangle",
        "origin": "igraph_atlas(7) — last 3-vertex entry, the triangle K_3",
        "algo": "atlas",
        "params": {"number": 7},
        "expected": {
            "vcount": 3,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 2]],
        },
    },
    {
        "case": "atlas_c_k4",
        "origin": "igraph_atlas(18) — last 4-vertex entry, the complete graph K_4",
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
        "case": "atlas_c_v6_null",
        "origin": "igraph_atlas(53) — first 6-vertex entry (null graph), cell boundary",
        "algo": "atlas",
        "params": {"number": 53},
        "expected": {
            "vcount": 6,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "atlas_c_k6_last_6v",
        "origin": "igraph_atlas(208) — last 6-vertex entry, the complete graph K_6",
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
        "case": "atlas_c_v7_null",
        "origin": "igraph_atlas(209) — first 7-vertex entry (null graph), cell boundary",
        "algo": "atlas",
        "params": {"number": 209},
        "expected": {
            "vcount": 7,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "atlas_c_k7_last_atlas_graph",
        "origin": "igraph_atlas(1252) — last entry in the atlas, the complete graph K_7",
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
        "case": "mycielskian_c_p3_one_iteration",
        "origin": "mycielskian(P_3, k=1) → 7v/9e; mirrors the recurrence on the smallest non-trivial path",
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
        "case": "mycielskian_c_c5_one_iteration",
        "origin": "mycielskian(C_5, k=1) → 11v/20e (= Grötzsch graph, since C_5 = M_3)",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (0, 3), (1, 2), (2, 4), (3, 4)],
            directed=False,
        ),
        "algo": "mycielskian",
        "params": {"k": 1},
        "expected": {
            "vcount": 11,
            "ecount": 20,
            "directed": False,
            "edges": [
                [0, 1], [0, 3], [1, 2], [2, 4], [3, 4],
                [0, 6], [1, 5], [0, 8], [3, 5], [1, 7],
                [2, 6], [2, 9], [4, 7], [3, 9], [4, 8],
                [5, 10], [6, 10], [7, 10], [8, 10], [9, 10],
            ],
        },
    },
    {
        "case": "mycielskian_c_null_two_iterations",
        "origin": "mycielskian(null, k=2) → P_2; promotes null→singleton (k=1) then singleton→P_2 (k=0)",
        "graph_factory": lambda: ig.Graph(n=0, edges=[], directed=False),
        "algo": "mycielskian",
        "params": {"k": 2},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
    {
        "case": "mycielskian_c_k0_identity",
        "origin": "mycielskian(K_3, k=0) → input unchanged (k=0 short-circuit)",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2), (0, 2)],
            directed=False,
        ),
        "algo": "mycielskian",
        "params": {"k": 0},
        "expected": {
            "vcount": 3,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [1, 2], [0, 2]],
        },
    },
]


PRUFER_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "from_prufer_c_seq_2323",
        "origin": "mirrors igraph_from_prufer.c fixture 1: prufer = [2,3,2,3] → 6-vertex tree (edges 2-0, 3-1, 4-2, 3-2, 5-3)",
        "algo": "from_prufer",
        "params": {"prufer": [2, 3, 2, 3]},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 2], [1, 3], [2, 4], [2, 3], [3, 5]],
        },
    },
    {
        "case": "from_prufer_c_seq_024110",
        "origin": "mirrors igraph_from_prufer.c fixture 2: prufer = [0,2,4,1,1,0] → 8-vertex tree (edges 3-0, 5-2, 4-2, 4-1, 6-1, 1-0, 7-0)",
        "algo": "from_prufer",
        "params": {"prufer": [0, 2, 4, 1, 1, 0]},
        "expected": {
            "vcount": 8,
            "ecount": 7,
            "directed": False,
            "edges": [[0, 3], [2, 5], [2, 4], [1, 4], [1, 6], [0, 1], [0, 7]],
        },
    },
    {
        "case": "from_prufer_c_empty",
        "origin": "mirrors igraph_from_prufer.c fixture 3: empty prufer → P_2 (single edge 1-0)",
        "algo": "from_prufer",
        "params": {"prufer": []},
        "expected": {
            "vcount": 2,
            "ecount": 1,
            "directed": False,
            "edges": [[0, 1]],
        },
    },
]


# ALGO-CL-001: vertex_coloring_greedy + is_vertex_coloring.
# The greedy coloring is heuristic-dependent but we can verify:
# 1. is_vertex_coloring on known valid/invalid colorings
# 2. vertex_coloring_greedy on known chromatic number graphs
COLORING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "coloring_c_is_valid_k3",
        "origin": "igraph coloring.c: valid 3-coloring of K3",
        "graph_factory": lambda: ig.Graph(n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False),
        "algo": "coloring",
        "params": {"check": "is_vertex_coloring", "colors": [0, 1, 2]},
        "expected": True,
    },
    {
        "case": "coloring_c_is_invalid_k3",
        "origin": "igraph coloring.c: invalid coloring of K3 (two adjacent same color)",
        "graph_factory": lambda: ig.Graph(n=3, edges=[(0, 1), (0, 2), (1, 2)], directed=False),
        "algo": "coloring",
        "params": {"check": "is_vertex_coloring", "colors": [0, 0, 1]},
        "expected": False,
    },
    {
        "case": "coloring_c_greedy_petersen_cn",
        "origin": "igraph coloring.c: Petersen graph (χ=3), CN heuristic — our impl achieves optimal 3 colors",
        "graph_factory": lambda: ig.Graph.Famous("petersen"),
        "algo": "coloring",
        "params": {"check": "greedy_valid", "heuristic": "colored_neighbors"},
        "expected": {"valid": True, "max_colors": 3},
    },
]


# ALGO-CL-002: maximum_cardinality_search + is_chordal.
CHORDAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "chordal_c_path4",
        "origin": "igraph is_chordal.c: path graph P_4 is chordal (no cycle of length >= 4)",
        "graph_factory": lambda: ig.Graph(n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False),
        "algo": "chordal",
        "params": {"check": "is_chordal"},
        "expected": {"chordal": True, "fill_in": []},
    },
    {
        "case": "chordal_c_cycle4_not_chordal",
        "origin": "igraph is_chordal.c: cycle C_4 is NOT chordal — missing chord",
        "graph_factory": lambda: ig.Graph(n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=False),
        "algo": "chordal",
        "params": {"check": "is_chordal"},
        "expected": {"chordal": False},
    },
]


# ALGO-CL-003: maximum_bipartite_matching.
MATCHING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "matching_c_is_valid",
        "origin": "igraph matching.c: valid matching on K_{2,2}",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 2), (0, 3), (1, 2), (1, 3)], directed=False,
        ),
        "algo": "matching",
        "params": {"check": "is_matching", "matching": [2, 3, 0, 1]},
        "expected": True,
    },
    {
        "case": "matching_c_is_invalid",
        "origin": "igraph matching.c: invalid matching (vertex 0 matched to 2 but vertex 2 not matched to 0)",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 2), (0, 3), (1, 2), (1, 3)], directed=False,
        ),
        "algo": "matching",
        "params": {"check": "is_matching", "matching": [2, 3, 1, 0]},
        "expected": False,
    },
]


# ALGO-LO-001: layout_circle + layout_star — deterministic layouts.
# layout_circle places vertices evenly on a unit circle.
# layout_star places one vertex at origin, rest on unit circle.
import math

LAYOUT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "layout_c_circle_4",
        "origin": "igraph layout_circle on 4-vertex graph — vertices at angles 0, π/2, π, 3π/2",
        "graph_factory": lambda: ig.Graph(n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False),
        "algo": "layout",
        "params": {"algorithm": "circle"},
        "expected": [
            [1.0, 0.0],
            [math.cos(math.pi / 2), math.sin(math.pi / 2)],
            [math.cos(math.pi), math.sin(math.pi)],
            [math.cos(3 * math.pi / 2), math.sin(3 * math.pi / 2)],
        ],
    },
    {
        "case": "layout_c_star_center0",
        "origin": "igraph layout_star on 4-vertex graph — center=0 at origin, rest on unit circle",
        "graph_factory": lambda: ig.Graph(n=4, edges=[(0, 1), (0, 2), (0, 3)], directed=False),
        "algo": "layout",
        "params": {"algorithm": "star", "center": 0},
        "expected": [
            [0.0, 0.0],
            [1.0, 0.0],
            [math.cos(2 * math.pi / 3), math.sin(2 * math.pi / 3)],
            [math.cos(4 * math.pi / 3), math.sin(4 * math.pi / 3)],
        ],
    },
]


# ALGO-SP-031: `igraph_get_all_simple_paths` — enumerate all simple paths.
# Reference test at references/igraph/tests/unit/igraph_get_all_simple_paths.c.
ALL_SIMPLE_PATHS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "all_simple_paths_c_maxlen3",
        "origin": "igraph_get_all_simple_paths.out: 6-vertex graph, from=0 to=5, maxlen=3",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 5), (0, 3), (3, 4), (4, 5), (3, 2), (3, 5)],
            directed=False,
        ),
        "algo": "all_simple_paths",
        "params": {"from": 0, "to": [5], "mode": "all", "min_len": -1, "max_len": 3, "max_results": -1},
        "expected": [[0, 1, 2, 5], [0, 3, 2, 5], [0, 3, 4, 5], [0, 3, 5]],
    },
    {
        "case": "all_simple_paths_c_minlen4",
        "origin": "igraph_get_all_simple_paths.out: 6-vertex graph, from=0 to=5, minlen=4",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 5), (0, 3), (3, 4), (4, 5), (3, 2), (3, 5)],
            directed=False,
        ),
        "algo": "all_simple_paths",
        "params": {"from": 0, "to": [5], "mode": "all", "min_len": 4, "max_len": -1, "max_results": -1},
        "expected": [[0, 1, 2, 3, 4, 5], [0, 1, 2, 3, 5]],
    },
]


# ALGO-SP-030: `igraph_path_length_hist` — all-pairs shortest-path length histogram.
# Reference test at references/igraph/tests/unit/igraph_path_length_hist.c.
PATH_LENGTH_HIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "path_length_hist_c_two_connected",
        "origin": "igraph_path_length_hist.out: two connected vertices undirected — hist=[1], unconnected=0",
        "graph_factory": lambda: ig.Graph(n=2, edges=[(0, 1)], directed=False),
        "algo": "path_length_hist",
        "params": {"directed": False},
        "expected": {"hist": [1.0], "unconnected": 0.0},
    },
    {
        "case": "path_length_hist_c_directed_graph_undirected_mode",
        "origin": "igraph_path_length_hist.out: 6-vertex directed graph, directed=false — hist=[6,3,1], unconnected=5",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 1), (1, 2), (1, 3), (2, 0), (2, 3), (3, 4), (3, 4)],
            directed=True,
        ),
        "algo": "path_length_hist",
        "params": {"directed": False},
        "expected": {"hist": [6.0, 3.0, 1.0], "unconnected": 5.0},
    },
    {
        "case": "path_length_hist_c_directed_graph_directed_mode",
        "origin": "igraph_path_length_hist.out: 6-vertex directed graph, directed=true — hist=[7,5,1], unconnected=17",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (0, 2), (1, 1), (1, 2), (1, 3), (2, 0), (2, 3), (3, 4), (3, 4)],
            directed=True,
        ),
        "algo": "path_length_hist",
        "params": {"directed": True},
        "expected": {"hist": [7.0, 5.0, 1.0], "unconnected": 17.0},
    },
]


# ALGO-PR-036: `igraph_trussness` — k-truss decomposition (per-edge trussness).
# Reference test at references/igraph/tests/unit/igraph_trussness.c.
# Expected values from igraph_trussness.out.
TRUSSNESS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "trussness_c_simple_graph",
        "origin": "igraph_trussness.out: simple 12-vertex graph (K5 core + K4 subcore + bridges)",
        "graph_factory": lambda: ig.Graph(
            n=12,
            edges=[
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4),
                (3, 6), (3, 11), (4, 5), (4, 6), (5, 6),
                (5, 7), (5, 8), (5, 9), (6, 7), (6, 10), (6, 11),
                (7, 8), (7, 9), (8, 9), (8, 10),
            ],
            directed=False,
        ),
        "algo": "trussness",
        "params": {},
        "expected": [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 4, 4, 4, 3, 2, 3, 4, 4, 4, 2],
    },
    {
        "case": "trussness_c_graph_with_loops",
        "origin": "igraph_trussness.out: same graph + 3 self-loops (0-0, 7-7, 5-5) — loops get trussness 2",
        "graph_factory": lambda: ig.Graph(
            n=12,
            edges=[
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4),
                (3, 6), (3, 11), (4, 5), (4, 6), (5, 6),
                (5, 7), (5, 8), (5, 9), (6, 7), (6, 10), (6, 11),
                (7, 8), (7, 9), (8, 9), (8, 10),
                (0, 0), (7, 7), (5, 5),
            ],
            directed=False,
        ),
        "algo": "trussness",
        "params": {},
        "expected": [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 3, 3, 3, 3, 3, 4, 4, 4, 3, 2, 3, 4, 4, 4, 2, 2, 2, 2],
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
    "count_isomorphisms_vf2": VF2_COUNT_MANIFEST,
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
    "bipartite_game_gnp": BIPARTITE_GAME_GNP_MANIFEST,
    "bipartite_game_gnm": BIPARTITE_GAME_GNM_MANIFEST,
    "iea_game": IEA_GAME_MANIFEST,
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
    "recent_degree_aging_game": RECENT_DEGREE_AGING_MANIFEST,
    "dot_product_game": DOT_PRODUCT_MANIFEST,
    "correlated_game": CORRELATED_MANIFEST,
    "correlated_pair_game": CORRELATED_PAIR_MANIFEST,
    "degree_sequence_game_configuration": DEGREE_SEQUENCE_CONFIG_MANIFEST,
    "degree_sequence_game_configuration_simple": DEGREE_SEQUENCE_CONFIG_SIMPLE_MANIFEST,
    "degree_sequence_game_edge_switching_simple": DEGREE_SEQUENCE_EDGE_SWITCHING_SIMPLE_MANIFEST,
    "degree_sequence_game_fast_heur_simple": DEGREE_SEQUENCE_FAST_HEUR_MANIFEST,
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
    "turan": TURAN_MANIFEST,
    "extended_chordal_ring": EXTENDED_CHORDAL_RING_MANIFEST,
    "adjacency": ADJACENCY_MANIFEST,
    "weighted_adjacency": WEIGHTED_ADJACENCY_MANIFEST,
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
    "trussness": TRUSSNESS_MANIFEST,
    "path_length_hist": PATH_LENGTH_HIST_MANIFEST,
    "all_simple_paths": ALL_SIMPLE_PATHS_MANIFEST,
    "layout": LAYOUT_MANIFEST,
    "coloring": COLORING_MANIFEST,
    "chordal": CHORDAL_MANIFEST,
    "matching": MATCHING_MANIFEST,
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
        # `community_to_membership` is a pure-function helper on a dendrogram
        # (merges matrix + leaf count + cut steps) and has no graph input.
        # The manifest entry carries the dendrogram directly; the graph block
        # is stubbed (only `n` matters, to carry the leaf count).
        if algo == "community_to_membership":
            nodes = int(entry["nodes"])
            payload = {
                "source": "c",
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
            # Pure helper on a membership vector — no graph input.
            membership = [int(c) for c in entry["membership"]]
            payload = {
                "source": "c",
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
            # Pure helper on two membership vectors — no graph input.
            comm1 = [int(c) for c in entry["comm1"]]
            comm2 = [int(c) for c in entry["comm2"]]
            payload = {
                "source": "c",
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
            # Pure helper on two membership vectors — returns asymmetric (d12, d21).
            comm1 = [int(c) for c in entry["comm1"]]
            comm2 = [int(c) for c in entry["comm2"]]
            payload = {
                "source": "c",
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
            "bipartite_game_gnp",
            "bipartite_game_gnm",
            "iea_game",
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
            "recent_degree_aging_game",
            "dot_product_game",
            "correlated_pair_game",
            "degree_sequence_game_configuration",
            "degree_sequence_game_configuration_simple",
            "degree_sequence_game_edge_switching_simple",
            "degree_sequence_game_fast_heur_simple",
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
            "turan",
            "extended_chordal_ring",
            "adjacency",
            "weighted_adjacency",
            "from_prufer",
            "tree_from_parent_vector",
            "lcf",
            "mycielski_graph",
            "famous",
            "atlas",
            "create",
            "triangular_lattice",
            "hexagonal_lattice",
        ):
            # Generators produce a graph from params alone — graph
            # payload is a placeholder, expected carries the structural
            # invariants the upstream examples assert
            # (`igraph_erdos_renyi_game_*` for ER, `igraph_barabasi_game`
            # for BA-BAG).
            payload = {
                "source": "c",
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
