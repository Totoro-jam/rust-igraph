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

# Shared `power_law_fit` datasets (the two built-in vectors from the igraph C
# unit test). See `_plfit_data.json`.
_PLFIT_DATA = json.loads((Path(__file__).parent / "_plfit_data.json").read_text())


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

# VF2 automorphism counts. R igraph exposes count_automorphisms(graph) (and
# the older graph.count.isomorphisms.vf2). The undirected ring(n) has 2n
# automorphisms and a path on 3 vertices has 2. Values are mathematical
# invariants identical across implementations and verified with
# python-igraph 0.11.9. Graphs are simple and loopless (VF2 requires it).
VF2_COUNT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_isomorphisms_vf2_R_ring4_undirected",
        "origin": "R igraph count_automorphisms(make_ring(4)) == 8 (4 rotations x 2 reflections)",
        "graph_factory": lambda: _ring(4),
        "algo": "count_isomorphisms_vf2",
        "params": {},
        "expected": 8,
    },
    {
        "case": "count_isomorphisms_vf2_R_path3",
        "origin": "R igraph count_automorphisms(make_lattice(c(3))) path 0-1-2 == 2 (identity + endpoint swap)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "count_isomorphisms_vf2",
        "params": {},
        "expected": 2,
    },
]

# VF2 subgraph self-count: count_subgraph_isomorphisms(g, g) in R igraph
# equals the automorphism count. Values are graph invariants verified with
# python-igraph 0.11.9. Simple, loopless graphs.
SUBISO_COUNT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_subisomorphisms_vf2_R_ring4_undirected",
        "origin": "R igraph count_subgraph_isomorphisms(make_ring(4), make_ring(4)) == 8 (4 rotations x 2 reflections)",
        "graph_factory": lambda: _ring(4),
        "algo": "count_subisomorphisms_vf2",
        "params": {},
        "expected": 8,
    },
    {
        "case": "count_subisomorphisms_vf2_R_path3",
        "origin": "R igraph count_subgraph_isomorphisms path 0-1-2 into itself == 2 (identity + endpoint swap)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "count_subisomorphisms_vf2",
        "params": {},
        "expected": 2,
    },
]

# BLISS automorphism counts. Authentic values verbatim from
# references/rigraph/tests/testthat/test-topology.R: count_automorphisms(g)
# returns a list whose $group_size is the order as a string ("20", "24", "4").
# R colours c(1,2,1,2) are colour classes {1,3} and {2,4}; we encode them
# 0-based as [0,1,0,1] (only the partition matters, not the labels).
COUNT_AUTOMORPHISMS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "count_automorphisms_R_ring10",
        "origin": "test-topology.R: count_automorphisms(make_ring(10))$group_size == \"20\" (dihedral group of the 10-cycle)",
        "graph_factory": lambda: _ring(10),
        "algo": "count_automorphisms",
        "params": {},
        "expected": 20,
    },
    {
        "case": "count_automorphisms_R_full4",
        "origin": "test-topology.R: count_automorphisms(make_full_graph(4))$group_size == \"24\" (4! permutations of K4)",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "count_automorphisms",
        "params": {},
        "expected": 24,
    },
    {
        "case": "count_automorphisms_R_full4_colored",
        "origin": "test-topology.R: count_automorphisms(make_full_graph(4), colors=c(1,2,1,2))$group_size == \"4\"",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "count_automorphisms",
        "params": {"colors": [0, 1, 0, 1]},
        "expected": 4,
    },
]

# BLISS automorphism generators. test-topology.R checks the (non-unique)
# generator lists directly; here we derive the group order (closure size) from
# the generators, which is the implementation-independent invariant. The
# generating sets in test-topology.R close to: make_ring(10) -> 20 (2 gens:
# reflection + rotation), make_full_graph(4) -> 24 (3 gens), coloured
# make_full_graph(4) c(1,2,1,2) -> 4 (2 gens).
AUTOMORPHISM_GROUP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "automorphism_group_R_ring10",
        "origin": "test-topology.R: automorphism_group(make_ring(10)) = {reflection, rotation}; closure order 20",
        "graph_factory": lambda: _ring(10),
        "algo": "automorphism_group",
        "params": {},
        "expected": 20,
    },
    {
        "case": "automorphism_group_R_full4",
        "origin": "test-topology.R: automorphism_group(make_full_graph(4)) generators close to S4, order 24",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "automorphism_group",
        "params": {},
        "expected": 24,
    },
    {
        "case": "automorphism_group_R_full4_colored",
        "origin": "test-topology.R: automorphism_group(make_full_graph(4), colors=c(1,2,1,2)) closure order 4",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "automorphism_group",
        "params": {"colors": [0, 1, 0, 1]},
        "expected": 4,
    },
]

# BLISS isomorphism yes/no. R igraph exposes isomorphic(g1, g2, method="bliss")
# and the colour-aware path via canonical_permutation(..., colors=). The verdict
# is a graph invariant (relabel-/implementation-independent); each expected
# value was verified against python-igraph 0.11.9 (= igraph C 0.10.16, the same
# engine rigraph links). R colours c(1,2,1,2) are colour classes encoded 0-based
# as [0,1,0,1].
ISOMORPHIC_BLISS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "isomorphic_bliss_R_ring6_relabel",
        "origin": "R igraph isomorphic(make_ring(6), relabelled make_ring(6), method=\"bliss\") == TRUE",
        "graph_factory": lambda: _ring(6),
        "algo": "isomorphic_bliss",
        "params": {
            "other": {
                "n": 6,
                "edges": [[0, 2], [2, 4], [4, 1], [1, 3], [3, 5], [5, 0]],
                "directed": False,
            }
        },
        "expected": True,
    },
    {
        "case": "isomorphic_bliss_R_ring6_vs_path6",
        "origin": "R igraph isomorphic(make_ring(6), make_lattice(c(6)), method=\"bliss\") == FALSE (cycle vs path)",
        "graph_factory": lambda: _ring(6),
        "algo": "isomorphic_bliss",
        "params": {
            "other": {
                "n": 6,
                "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]],
                "directed": False,
            }
        },
        "expected": False,
    },
    {
        "case": "isomorphic_bliss_R_full4",
        "origin": "R igraph isomorphic(make_full_graph(4), make_full_graph(4), method=\"bliss\") == TRUE",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "isomorphic_bliss",
        "params": {
            "other": {
                "n": 4,
                "edges": [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]],
                "directed": False,
            }
        },
        "expected": True,
    },
    {
        "case": "isomorphic_bliss_R_full4_colored_match",
        "origin": "R igraph BLISS colour-aware: make_full_graph(4) with colours c(1,2,1,2) vs same == TRUE",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "isomorphic_bliss",
        "params": {
            "other": {
                "n": 4,
                "edges": [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]],
                "directed": False,
            },
            "colors1": [0, 1, 0, 1],
            "colors2": [0, 1, 0, 1],
        },
        "expected": True,
    },
    {
        "case": "isomorphic_bliss_R_ring10_color_mismatch",
        "origin": "R igraph BLISS colour-aware: make_ring(10) colour distributions {1x1,9x0} vs {10x0} == FALSE",
        "graph_factory": lambda: _ring(10),
        "algo": "isomorphic_bliss",
        "params": {
            "other": {
                "n": 10,
                "edges": [
                    [0, 1], [1, 2], [2, 3], [3, 4], [4, 5],
                    [5, 6], [6, 7], [7, 8], [8, 9], [9, 0],
                ],
                "directed": False,
            },
            "colors1": [1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            "colors2": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        },
        "expected": False,
    },
]

# Generic isomorphic(g1, g2) dispatcher (R auto-selects a method).
ISOMORPHIC_GENERIC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "isomorphic_R_ring6_relabel",
        "origin": "R igraph isomorphic(make_ring(6), relabelled make_ring(6)) == TRUE",
        "graph_factory": lambda: _ring(6),
        "algo": "isomorphic",
        "params": {
            "other": {
                "n": 6,
                "edges": [[0, 2], [2, 4], [4, 1], [1, 3], [3, 5], [5, 0]],
                "directed": False,
            }
        },
        "expected": True,
    },
    {
        "case": "isomorphic_R_ring6_vs_path6",
        "origin": "R igraph isomorphic(make_ring(6), make_lattice(c(6))) == FALSE (cycle vs path)",
        "graph_factory": lambda: _ring(6),
        "algo": "isomorphic",
        "params": {
            "other": {
                "n": 6,
                "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]],
                "directed": False,
            }
        },
        "expected": False,
    },
    {
        "case": "isomorphic_R_full4_vs_full5",
        "origin": "R igraph isomorphic(make_full_graph(4), make_full_graph(5)) == FALSE (vcount differs)",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "isomorphic",
        "params": {
            "other": {
                "n": 5,
                "edges": [
                    [0, 1], [0, 2], [0, 3], [0, 4], [1, 2],
                    [1, 3], [1, 4], [2, 3], [2, 4], [3, 4],
                ],
                "directed": False,
            }
        },
        "expected": False,
    },
    {
        "case": "isomorphic_R_directed_ring4",
        "origin": "R igraph isomorphic(make_ring(4, directed=TRUE), rotated copy) == TRUE",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3), (3, 0)], directed=True
        ),
        "algo": "isomorphic",
        "params": {
            "other": {
                "n": 4,
                "edges": [[1, 2], [2, 3], [3, 0], [0, 1]],
                "directed": True,
            }
        },
        "expected": True,
    },
]

# Generic subgraph_isomorphic(pattern, target). The conformance harness stores
# the target as `graph` and the pattern as `params.other` (subisomorphic's
# graph1=target, graph2=pattern convention).
SUBISOMORPHIC_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "subisomorphic_R_triangle_in_full4",
        "origin": "R igraph subgraph_isomorphic(make_ring(3), make_full_graph(4)) == TRUE (triangle embeds in K4)",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False),
        "algo": "subisomorphic",
        "params": {
            "other": {
                "n": 3,
                "edges": [[0, 1], [1, 2], [2, 0]],
                "directed": False,
            }
        },
        "expected": True,
    },
    {
        "case": "subisomorphic_R_triangle_in_path3",
        "origin": "R igraph subgraph_isomorphic(make_ring(3), make_lattice(c(3))) == FALSE (path has no 3-clique)",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "subisomorphic",
        "params": {
            "other": {
                "n": 3,
                "edges": [[0, 1], [1, 2], [2, 0]],
                "directed": False,
            }
        },
        "expected": False,
    },
]

# LAD subgraph isomorphism. rigraph exposes this as
# `subgraph_isomorphic(pattern, target, method = "lad", induced=, domains=)`
# and `graph.get.subisomorphisms.vf2`-style map listing via
# `subisomorphic_lad` internals; both run the shared igraph C 0.10.16
# library, so the verdicts/maps are identical to the C example oracle
# (examples/simple/igraph_subisomorphic_lad.out: 20/4/1). Map lists sorted
# to compare as sets.
_LAD_R_TARGET_FACTORY = lambda: ig.Graph(
    n=9,
    edges=[
        (0, 1), (0, 4), (0, 6),
        (1, 4), (1, 2),
        (2, 3),
        (3, 4), (3, 5), (3, 7), (3, 8),
        (4, 5), (4, 6),
        (5, 6), (5, 8),
        (7, 8),
    ],
    directed=False,
)
_LAD_R_PATTERN = {
    "n": 5,
    "edges": [[0, 1], [0, 4], [1, 4], [1, 2], [2, 3], [3, 4]],
    "directed": False,
}

SUBISOMORPHIC_LAD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "subisomorphic_lad_R_example_mono",
        "origin": "R igraph subgraph_isomorphic(pattern, target, method='lad') == TRUE (example graphs, monomorphism)",
        "graph_factory": _LAD_R_TARGET_FACTORY,
        "algo": "subisomorphic_lad",
        "params": {"other": _LAD_R_PATTERN, "induced": False, "domains": None},
        "expected": True,
    },
    {
        "case": "subisomorphic_lad_R_triangle_in_path3",
        "origin": "R igraph subgraph_isomorphic(make_ring(3), make_lattice(c(3)), method='lad') == FALSE (path has no 3-clique)",
        "graph_factory": lambda: ig.Graph(n=3, edges=[(0, 1), (1, 2)], directed=False),
        "algo": "subisomorphic_lad",
        "params": {
            "other": {"n": 3, "edges": [[0, 1], [1, 2], [2, 0]], "directed": False},
            "induced": False,
            "domains": None,
        },
        "expected": False,
    },
]

GET_SUBISOMORPHISMS_LAD_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "get_subisomorphisms_lad_R_example_induced",
        "origin": "R igraph subisomorphic_lad(induced=TRUE) — 4 induced embeddings (example graphs, shared C 0.10.16, sorted)",
        "graph_factory": _LAD_R_TARGET_FACTORY,
        "algo": "get_subisomorphisms_lad",
        "params": {"other": _LAD_R_PATTERN, "induced": True, "domains": None},
        "expected": [
            [0, 1, 2, 3, 4], [0, 4, 3, 2, 1], [5, 3, 2, 1, 4], [5, 4, 1, 2, 3],
        ],
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

# ALGO-PR-018: is_perfect. rigraph exposes `is_perfect(graph)`
# (R/aaa-auto.R is_perfect_impl → R_igraph_is_perfect, same C entry
# point igraph_is_perfect). Verdicts mirror the upstream C unit test;
# graphs serialized via python-igraph for the shared fixture format.
IS_PERFECT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "is_perfect_R_c5_false",
        "origin": "rigraph is_perfect(make_ring(5)) — the 5-cycle is imperfect (minimal odd hole)",
        "graph_factory": lambda: ig.Graph.Ring(n=5, circular=True),
        "algo": "is_perfect",
        "params": {},
        "expected": False,
    },
    {
        "case": "is_perfect_R_house_true",
        "origin": "rigraph is_perfect(make_graph('House')) — the House graph is perfect",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (0, 2), (1, 3), (2, 3), (2, 4), (3, 4)],
            directed=False,
        ),
        "algo": "is_perfect",
        "params": {},
        "expected": True,
    },
    {
        "case": "is_perfect_R_paley9_true",
        "origin": "rigraph is_perfect(Paley(9)) — self-complementary perfect graph",
        "graph_factory": lambda: ig.Graph(
            n=9,
            edges=[
                (0, 1), (0, 3), (0, 6), (0, 2), (1, 2), (1, 4), (1, 7),
                (2, 5), (2, 8), (3, 4), (3, 5), (3, 6), (4, 5), (4, 7),
                (5, 8), (6, 7), (7, 8), (6, 8),
            ],
            directed=False,
        ),
        "algo": "is_perfect",
        "params": {},
        "expected": True,
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

# ALGO-PR-067: generic value-based assortativity. rigraph's
# `assortativity(graph, values, ..., values.in=NULL, directed=, normalized=)`
# shares the igraph C core, so the coefficient is identical to the value
# computed live via python-igraph here (Rscript is unavailable in this
# environment). The R API has no edge-weight argument; fixtures are
# unweighted, mirroring the R-style path/directed smoke cases.
ASSORT_VAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "assortativity_values_R_path3_norm",
        "origin": "test-aaa-auto.R-style — assortativity(path_graph(3), values=c(1,2,1)); same C core, value via python-igraph 0.11.9",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "values": [1.0, 2.0, 1.0],
        "values_in": None,
        "directed": False,
        "normalized": True,
    },
    {
        "case": "assortativity_values_R_directed_hub_reuse",
        "origin": "test-aaa-auto.R-style — assortativity(directed hub, values=1:5, values.in=NULL); reuses values for both endpoints; same C core, value via python-igraph 0.11.9",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (0, 2), (0, 3), (1, 3), (2, 3), (4, 3)],
            directed=True,
        ),
        "values": [1.0, 2.0, 3.0, 4.0, 5.0],
        "values_in": None,
        "directed": True,
        "normalized": True,
    },
]

ASTAR_R_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "astar_R_zachary_unweighted",
        "origin": "rigraph shares igraph C core; computed live via py-igraph get_shortest_path_astar (null heuristic)",
        "graph_factory": lambda: ig.Graph.Famous("Zachary"),
        "from": 0,
        "to": 33,
        "weights": None,
        "mode": "all",
    },
    {
        "case": "astar_R_ring5_weighted",
        "origin": "rigraph shares igraph C core; computed live via py-igraph (null heuristic)",
        "graph_factory": lambda: ig.Graph.Ring(5),
        "from": 0,
        "to": 3,
        "weights": [1.0, 2.0, 3.0, 1.0, 1.0],
        "mode": "all",
    },
]

ALL_SP_DIJKSTRA_R_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "all_sp_dij_r_ring6_unit",
        "origin": "rigraph shares igraph C core; ring(6) unit weights from vertex 0",
        "graph_factory": lambda: ig.Graph.Ring(6),
        "source": 0,
        "weights": None,
    },
    {
        "case": "all_sp_dij_r_diamond_equal",
        "origin": "rigraph shares igraph C core; diamond equal weights",
        "graph_factory": lambda: ig.Graph(4, [(0, 1), (0, 2), (1, 3), (2, 3)], directed=False),
        "source": 0,
        "weights": [1.0, 1.0, 1.0, 1.0],
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


# R-igraph exposes `rich_club_sequence_impl()` (auto-generated wrapper
# around `igraph_rich_club_sequence`). The first fixture below mirrors
# the upstream snapshot test in
# references/rigraph/tests/testthat/test-aaa-auto.R Test "132."
# (P_3 path, vertex_order=1:3, normalized=true default), whose
# Output line in `_snaps/aaa-auto.md:3073` is
# `[1] 0.6666667 1.0000000 NaN`. The R `vertex_order=1:3` is 1-indexed
# and translates to the 0-indexed `[0, 1, 2]` the Rust function
# expects. The second fixture mirrors the second `expect_snapshot`
# in the same test (`normalized=FALSE, loops=TRUE, directed=FALSE`),
# whose output is `[1] 2 1 0` — raw edge counts at each peel step.
RICH_CLUB_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "rich_club_r_path3_inorder_normalized",
        "origin": "references/rigraph/tests/testthat/_snaps/aaa-auto.md:3073 — "
        "P_3 path_graph(3), vertex_order=1:3, normalized default",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [0, 1, 2],
            "normalized": True,
            "loops": False,
            "directed": False,
        },
        "expected": [2 / 3, 1.0, None],
    },
    {
        "case": "rich_club_r_path3_inorder_unnormalized_loops",
        "origin": "references/rigraph/tests/testthat/_snaps/aaa-auto.md:3081 — "
        "P_3 path_graph(3), vertex_order=1:3, normalized=FALSE, loops=TRUE, directed=FALSE",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=False
        ),
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [0, 1, 2],
            "normalized": False,
            "loops": True,
            "directed": False,
        },
        "expected": [2.0, 1.0, 0.0],
    },
    {
        # Hand-derived companion: K_4, peel by reversed degree-tied
        # order [3, 2, 1, 0]. Subgraph edge counts:
        #   i=0: full K_4, 6 edges, max C(4,2)=6 → 1.0
        #   i=1: K_3 on {2,1,0}, 3 edges, max 3 → 1.0
        #   i=2: edge (1,0),     1 edge,  max 1 → 1.0
        #   i=3: single vertex {0}, 0 edges, max 0 → NaN
        "case": "rich_club_r_k4_reverse_order_normalized",
        "origin": "hand-derived (R-style) — K_4 normalized, vertex_order=[3,2,1,0] → "
        "[1.0, 1.0, 1.0, NaN]",
        "graph_factory": lambda: ig.Graph.Full(n=4, directed=False, loops=False),
        "algo": "rich_club_sequence",
        "params": {
            "vertex_order": [3, 2, 1, 0],
            "normalized": True,
            "loops": False,
            "directed": False,
        },
        "expected": [1.0, 1.0, 1.0, None],
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

# ALGO-FL-002: max_flow_value. Mirrors rigraph's `max_flow(g, source,
# target, capacity)` — the R bindings expose only the full `max_flow`
# (returning `value`, `flow`, `cut`, ...), and the unit test in
# rigraph tests/testthat/test-flow.R:111-128 checks `flow$value` on a
# 6-vertex directed graph with explicit capacities. Edges (1-indexed
# in R) translate to 0-indexed: (0,2,3), (2,3,1), (3,1,2), (0,4,1),
# (4,5,2), (5,1,10). Two vertex-disjoint augmenting paths each
# delivering 1 unit of flow → max-flow value = 2.
MAXFLOW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "maxflow_r_directed_6v_weighted",
        "origin": "rigraph tests/testthat/test-flow.R:111-128 — "
        "`max_flow(g_ring_acyc, source='1', target='2')$value == 2`",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 2), (2, 3), (3, 1), (0, 4), (4, 5), (5, 1)],
            directed=True,
        ),
        "graph_weights": [3.0, 1.0, 2.0, 1.0, 2.0, 10.0],
        "algo": "max_flow_value",
        "params": {"source": 0, "target": 1, "use_capacity": True},
        "expected": 2.0,
    },
]

# ALGO-FL-010: st_mincut_value. rigraph exposes `min_cut(g, source,
# target, capacity, value.only=TRUE)` (R/flow.R:386-437) which
# dispatches to `st_mincut_value_impl` whenever both source and target
# are supplied — verbatim mirror of `igraph_st_mincut_value`. By
# max-flow / min-cut duality the value equals `max_flow(g, source,
# target, capacity)$value`; we therefore replay the same 6-vertex
# directed fixture from test-flow.R as the maxflow manifest, with
# expected mincut = 2.
ST_MINCUT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_mincut_r_directed_6v_weighted",
        "origin": "rigraph tests/testthat/test-flow.R:111-128 — by duality "
        "`min_cut(g_ring_acyc, source='1', target='2', "
        "capacity=c(3,1,2,1,2,10)) == 2`",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 2), (2, 3), (3, 1), (0, 4), (4, 5), (5, 1)],
            directed=True,
        ),
        "graph_weights": [3.0, 1.0, 2.0, 1.0, 2.0, 10.0],
        "algo": "st_mincut_value",
        "params": {"source": 0, "target": 1, "use_capacity": True},
        "expected": 2.0,
    },
]


# ALGO-FL-011: st_edge_connectivity. rigraph exposes
# `edge_connectivity(graph, source, target)` which dispatches to
# `igraph_st_edge_connectivity` when both endpoints are supplied. Test
# tests/testthat/test-flow.R:146-154 asserts:
#   - K_5 undirected: `edge_connectivity(g_full, source=1, target=2)
#     == 4`
#   - directed acyclic ring 1→2→3→4→5: `edge_connectivity(g_path,
#     source=1, target=3) == 1`
# (R is 1-indexed; we shift to 0-indexed here.)
ST_EDGE_CONN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_edge_conn_r_full_5v",
        "origin": "rigraph tests/testthat/test-flow.R:148 "
        "(`edge_connectivity(make_full_graph(5), source=1, target=2)` "
        "== 4); R 1-indexed source=1,target=2 → Rust 0-indexed "
        "source=0, target=1",
        "graph_factory": lambda: ig.Graph.Full(5, directed=False),
        "algo": "st_edge_connectivity",
        "params": {"source": 0, "target": 1},
        "expected": 4,
    },
    {
        "case": "st_edge_conn_r_directed_path_5v",
        "origin": "rigraph tests/testthat/test-flow.R:153 "
        "(`edge_connectivity(make_ring(5, directed=TRUE, "
        "circular=FALSE), source=1, target=3)` == 1); R 1-indexed "
        "source=1,target=3 → Rust 0-indexed source=0, target=2",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=True
        ),
        "algo": "st_edge_connectivity",
        "params": {"source": 0, "target": 2},
        "expected": 1,
    },
]


# ALGO-FL-012: edge_disjoint_paths. rigraph exposes
# `edge_disjoint_paths(graph, source, target)` which dispatches to
# `igraph_edge_disjoint_paths`. Test tests/testthat/test-flow.R:183-189
# asserts:
#   - K_5 undirected: `edge_disjoint_paths(g_full, source=1, target=2)
#     == 4`
#   - directed acyclic ring 1→2→3→4→5: `edge_disjoint_paths(g_path,
#     source=1, target=3) == 1`
# (R is 1-indexed; we shift to 0-indexed here.)
ED_PATHS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "edge_disjoint_paths_r_full_5v",
        "origin": "rigraph tests/testthat/test-flow.R:185 "
        "(`edge_disjoint_paths(make_full_graph(5), source=1, target=2)` "
        "== 4); R 1-indexed source=1,target=2 → Rust 0-indexed "
        "source=0, target=1",
        "graph_factory": lambda: ig.Graph.Full(5, directed=False),
        "algo": "edge_disjoint_paths",
        "params": {"source": 0, "target": 1},
        "expected": 4,
    },
    {
        "case": "edge_disjoint_paths_r_directed_path_5v",
        "origin": "rigraph tests/testthat/test-flow.R:188 "
        "(`edge_disjoint_paths(make_ring(5, directed=TRUE, "
        "circular=FALSE), source=1, target=3)` == 1); R 1-indexed "
        "source=1,target=3 → Rust 0-indexed source=0, target=2",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=True
        ),
        "algo": "edge_disjoint_paths",
        "params": {"source": 0, "target": 2},
        "expected": 1,
    },
]


# ALGO-FL-013: st_vertex_connectivity. rigraph exposes
# `vertex_connectivity(graph, source, target)` which dispatches to
# `st_vertex_connectivity_impl` (R/aaa-auto.R:11598) which defaults
# `neighbors = "number_of_nodes"`. Test tests/testthat/test-flow.R:130-139
# asserts:
#   - 5v circular ring (`make_ring(5, circular=TRUE)`), source=1, target=4
#     → vc == 2 (R 1-indexed → Rust source=0, target=3); no direct edge so
#     mode-independent; two disjoint paths 0→1→2→3 and 0→4→3.
# Plus a K_6 directed IGNORE sanity that overlaps with the C fixture.
ST_VCONN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_vconn_r_ring5_circular",
        "origin": "rigraph tests/testthat/test-flow.R:138 "
        "(vertex_connectivity(make_ring(5, circular=TRUE), source=1, "
        "target=4) == 2); R 1-indexed → Rust source=0, target=3; "
        "no direct edge so any mode returns 2",
        "graph_factory": lambda: ig.Graph.Ring(5, directed=False, circular=True),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 3, "mode": "number_of_nodes"},
        "expected": 2,
    },
    {
        "case": "st_vconn_r_full_6v_undirected_ignore",
        "origin": "rigraph cross-check using K_6 undirected (same fixture "
        "structure as the C `igraph_st_vertex_connectivity.c:57` case but "
        "exposed via rigraph's `vertex_connectivity` wrapper): vc(0,1) "
        "with mode IGNORE returns 4",
        "graph_factory": lambda: ig.Graph.Full(6, directed=False, loops=False),
        "algo": "st_vertex_connectivity",
        "params": {"source": 0, "target": 1, "mode": "ignore"},
        "expected": 4,
    },
]


# ALGO-FL-014: vertex_disjoint_paths. rigraph exposes
# `vertex_disjoint_paths(graph, source, target)` via
# R/aaa-auto.R wrapping `igraph_vertex_disjoint_paths`. Tests at
# tests/testthat/test-flow.R:201-207 cover two minimal cases:
#   - make_full_graph(5), source=1, target=2 (R 1-indexed) →
#     vdp(0, 1) = 4 on undirected K_5 (all four other vertices give
#     four internally vertex-disjoint paths plus the direct edge,
#     but igraph subtracts the direct-edge count under `Ignore`
#     and adds it back → 1 direct + 3 detours = 4 in our convention).
#   - make_ring(5, directed=TRUE, circular=FALSE), source=1, target=3 →
#     vdp(0, 2) = 1 on the directed path 0→1→2→3→4 (single chain).
VDP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "vdp_r_full_5v_0_to_1",
        "origin": "rigraph tests/testthat/test-flow.R:202-203 "
        "(vertex_disjoint_paths(make_full_graph(5), source=1, "
        "target=2) == 4); R 1-indexed → Rust source=0, target=1; "
        "undirected K_5 → 4 disjoint paths",
        "graph_factory": lambda: ig.Graph.Full(5, directed=False, loops=False),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 0, "target": 1},
        "expected": 4,
    },
    {
        "case": "vdp_r_directed_path5_0_to_2",
        "origin": "rigraph tests/testthat/test-flow.R:205-206 "
        "(vertex_disjoint_paths(make_ring(5, directed=TRUE, "
        "circular=FALSE), source=1, target=3) == 1); R 1-indexed → "
        "Rust source=0, target=2; directed path 0→1→2→3→4 has a "
        "single 0→2 walk so vdp = 1",
        "graph_factory": lambda: ig.Graph.Ring(5, directed=True, circular=False),
        "algo": "vertex_disjoint_paths",
        "params": {"source": 0, "target": 2},
        "expected": 1,
    },
]


# ALGO-FL-015: vertex_connectivity (global cohesion). rigraph exposes
# `vertex_connectivity(graph)` (no source/target → global) and the
# alias `cohesion(graph)` via R/aaa-auto.R wrapping
# `igraph_vertex_connectivity`. Tests at tests/testthat/test-flow.R:130-138
# cover three global cases:
#   - make_ring(5, circular=FALSE)            → vc = 1 (undirected path)
#   - make_graph(edges=c(1,2,3,4),            → vc = 0 (two isolated edges)
#                directed=FALSE)
#   - make_ring(5, circular=TRUE)             → vc = 2 (undirected 5-cycle)
#                                                (R uses source/target=2;
#                                                 we record the global form
#                                                 because both kappa(s,t)
#                                                 evaluate to 2 on the ring
#                                                 and the global min agrees)
# Also cover cohesion alias hits in tests/testthat/test-cohesion.R:
#   - karate → cohesion = 1, kite → cohesion = 1, camp → cohesion = 2.
# These named-graph fixtures are not bundled here; the three ring/path
# cases give branch coverage for the cheap short-circuits and the
# pairwise loop.
VCONN_GLOBAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "vconn_r_path5_undirected_returns_one",
        "origin": "rigraph tests/testthat/test-flow.R:131-132 "
        "(vertex_connectivity(make_ring(5, circular=FALSE)) == 1); "
        "undirected path 0-1-2-3-4 has endpoints of degree 1 → cheap "
        "min-degree short-circuit returns 1.",
        "graph_factory": lambda: ig.Graph.Ring(5, circular=False, directed=False),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 1,
    },
    {
        "case": "vconn_r_two_isolated_edges_undirected_returns_zero",
        "origin": "rigraph tests/testthat/test-flow.R:134-135 "
        "(vertex_connectivity(make_graph(edges=c(1,2,3,4), "
        "directed=FALSE)) == 0); two components {0,1} and {2,3} → "
        "graph not connected → cheap connectedness short-circuit "
        "returns 0.",
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
        "case": "vconn_r_ring5_undirected_returns_two",
        "origin": "rigraph tests/testthat/test-flow.R:137-138 "
        "(s-t form: vertex_connectivity(make_ring(5, circular=TRUE), "
        "source=1, target=4) == 2); the global vertex_connectivity of "
        "an undirected 5-cycle is 2 (every internal pair needs two "
        "vertices removed). Captured here as the global form so the "
        "FL-015 conformance covers the pairwise-loop path (no cheap "
        "short-circuit fires: connected, min-degree=2, not complete).",
        "graph_factory": lambda: ig.Graph.Ring(5, circular=True, directed=False),
        "algo": "vertex_connectivity",
        "params": {"checks": True},
        "expected": 2,
    },
]


# ALGO-FL-016: edge_connectivity (global adhesion). rigraph exposes
# `edge_connectivity(graph)` (no source/target → global) and the alias
# `adhesion(graph)` via R/aaa-auto.R wrapping `igraph_edge_connectivity`.
# Tests at tests/testthat/test-flow.R cover three global cases mirroring
# the vertex_connectivity layout:
#   - make_ring(5, circular=FALSE)            → ec = 1 (undirected path:
#                                                end edges are bridges)
#   - make_graph(edges=c(1,2,3,4),            → ec = 0 (two isolated
#                directed=FALSE)               edges → graph disconnected)
#   - make_ring(5, circular=TRUE)             → ec = 2 (undirected 5-cycle
#                                                — every cut must remove
#                                                two edges; no cheap
#                                                short-circuit fires:
#                                                connected, min-degree=2,
#                                                and edge_connectivity has
#                                                no complete-graph shortcut
#                                                so the fixed-vertex loop
#                                                runs)
ECONN_GLOBAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "econn_r_path5_undirected_returns_one",
        "origin": "rigraph tests/testthat/test-flow.R "
        "(edge_connectivity(make_ring(5, circular=FALSE)) == 1); "
        "undirected path 0-1-2-3-4 — every internal edge is a bridge — "
        "so the cheap min-degree short-circuit returns 1 immediately.",
        "graph_factory": lambda: ig.Graph.Ring(5, circular=False, directed=False),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 1,
    },
    {
        "case": "econn_r_two_isolated_edges_undirected_returns_zero",
        "origin": "rigraph tests/testthat/test-flow.R "
        "(edge_connectivity(make_graph(edges=c(1,2,3,4), "
        "directed=FALSE)) == 0); two components {0,1} and {2,3} → graph "
        "not connected → cheap connectedness short-circuit returns 0.",
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
        "case": "econn_r_ring5_undirected_returns_two",
        "origin": "rigraph tests/testthat/test-flow.R "
        "(edge_connectivity(make_ring(5, circular=TRUE)) == 2); "
        "undirected 5-cycle — no cheap short-circuit fires (connected, "
        "min-degree=2, no complete-graph shortcut for edge_connectivity), "
        "so the fixed-vertex loop runs and yields the global min edge "
        "cut value of 2.",
        "graph_factory": lambda: ig.Graph.Ring(5, circular=True, directed=False),
        "algo": "edge_connectivity",
        "params": {"checks": True},
        "expected": 2,
    },
]


# ALGO-FL-017: mincut_value — weighted global minimum-cut value.
# rigraph exposes `min_cut(graph, capacity = NULL)` returning the
# numeric mincut. We pin three rigraph-style fixtures: undirected ring
# with unit caps (matches edge_connectivity), undirected ring with a
# single weighted bridge edge, and a directed cycle with mixed arc
# capacities.
MINCUT_VALUE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mincut_r_ring5_undirected_unit_caps_returns_two",
        "origin": "rigraph tests/testthat/test-flow.R style: "
        "min_cut(make_ring(5)) == 2 — unit-capacity ring lambda(C_5) = 2.",
        "graph_factory": lambda: ig.Graph.Ring(5, circular=True, directed=False),
        "algo": "mincut_value",
        "params": {"capacity": None},
        "expected": 2.0,
    },
    {
        "case": "mincut_r_ring5_undirected_one_weak_edge",
        "origin": "rigraph tests/testthat/test-flow.R style: "
        "min_cut(g, capacity=c(10, 10, 0.5, 10, 10)) — single 0.5 "
        "bridge edge. Cheapest 2-edge cut is 0.5 + 10 = 10.5 (any other "
        "non-bridge pair costs ≥ 20).",
        "graph_factory": lambda: ig.Graph.Ring(5, circular=True, directed=False),
        "graph_weights": [10.0, 10.0, 0.5, 10.0, 10.0],
        "algo": "mincut_value",
        "params": {"capacity": [10.0, 10.0, 0.5, 10.0, 10.0]},
        "expected": 10.5,
    },
    {
        "case": "mincut_r_directed_3cycle_weighted",
        "origin": "rigraph tests/testthat/test-flow.R style: "
        "min_cut on directed 3-cycle with capacities c(2, 0.5, 3). "
        "The bottleneck arc 1→2 with weight 0.5 dominates the "
        "fixed-vertex iteration ⇒ mincut_value = 0.5.",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2), (2, 0)], directed=True
        ),
        "graph_weights": [2.0, 0.5, 3.0],
        "algo": "mincut_value",
        "params": {"capacity": [2.0, 0.5, 3.0]},
        "expected": 0.5,
    },
]

# ALGO-FL-018: st_mincut — full source/sink partition. rigraph exposes
# `min_cut(graph, source=s, target=t, capacity=NULL, value.only=FALSE)`
# which returns a list of $value $cut $partition1 $partition2. We pin
# three fixtures: a unit-cap single arc, a unit-cap two-parallel-paths
# diamond (value pinned only — multiple optimal cuts), and an
# undirected K_4 (value pinned only — multiple isolation cuts of the
# same minimum cost).
ST_MINCUT_PARTITION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "st_mincut_r_single_directed_edge_unit",
        "origin": "rigraph tests/testthat/test-flow.R style: "
        "min_cut(make_graph(c(1,2), directed=TRUE), source=1, target=2)"
        " ⇒ value=1, cut=c(1), partition1=c(1), partition2=c(2).",
        "graph_factory": lambda: ig.Graph(n=2, edges=[(0, 1)], directed=True),
        "algo": "st_mincut",
        "params": {"source": 0, "target": 1, "capacity": None},
        "expected": {
            "value": 1.0,
            "cut": [0],
            "partition": [0],
            "partition2": [1],
        },
    },
    {
        "case": "st_mincut_r_two_parallel_paths_unit_caps_value_only",
        "origin": "rigraph tests/testthat/test-flow.R style: "
        "min_cut on directed diamond (edges (1,2),(2,4),(1,3),(3,4)) "
        "from 1 to 4 with unit caps ⇒ value=2; multiple optimal cuts "
        "exist (e.g. {(1,2),(1,3)} or {(2,4),(3,4)}) so cut / "
        "partition are not pinned.",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 3), (0, 2), (2, 3)], directed=True
        ),
        "algo": "st_mincut",
        "params": {"source": 0, "target": 3, "capacity": None},
        "expected": {"value": 2.0},
    },
    {
        "case": "st_mincut_r_undirected_k4_unit_caps_value_only",
        "origin": "rigraph tests/testthat/test-flow.R style: "
        "min_cut(make_full_graph(4, directed=FALSE), source=1, "
        "target=2) on K_4 unit caps ⇒ value=3 (single-vertex "
        "isolation cuts are optimal; cut / partition non-unique).",
        "graph_factory": lambda: ig.Graph.Full(4, directed=False, loops=False),
        "algo": "st_mincut",
        "params": {"source": 0, "target": 1, "capacity": None},
        "expected": {"value": 3.0},
    },
]

# ALGO-FL-031: all_st_cuts. rigraph exposes
# `all_st_cuts(graph, source, target)` returning a list of cut
# structures. NOTE rigraph's API is 1-based for vertex / edge ids, but
# our stored `expected` values are the 0-based igraph-core ids the Rust
# port uses; values here are computed with python-igraph 0.11.9 (the
# shared oracle) which is already 0-based. `expected` is the canonical
# collection: `partition1s` aligned with `cuts`, sorted by
# (partition, cut) so the order is stable.
ALL_ST_CUTS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "all_st_cuts_r_single_directed_edge",
        "origin": "rigraph all_st_cuts(make_graph(c(1,2), directed=TRUE), "
        "source=1, target=2) ⇒ a single cut. Stored 0-based: "
        "partition1=[0], cut edge=[0] (R reports source=1, edge=1).",
        "graph_factory": lambda: ig.Graph(n=2, edges=[(0, 1)], directed=True),
        "algo": "all_st_cuts",
        "params": {"source": 0, "target": 1},
        "expected": {
            "partition1s": [[0]],
            "cuts": [[0]],
        },
    },
    {
        "case": "all_st_cuts_r_directed_k4",
        "origin": "rigraph all_st_cuts(make_full_graph(4, directed=TRUE), "
        "source=1, target=4) on directed K_4 ⇒ 4 cuts. Stored 0-based "
        "(R uses 1-based source=1 target=4); edge ids follow the K_4 "
        "insertion order (0,1)(0,2)(0,3)(1,0)(1,2)(1,3)(2,0)(2,1)(2,3)"
        "(3,0)(3,1)(3,2).",
        "graph_factory": lambda: ig.Graph.Full(4, directed=True, loops=False),
        "algo": "all_st_cuts",
        "params": {"source": 0, "target": 3},
        "expected": {
            "partition1s": [[0], [0, 1], [0, 1, 2], [0, 2]],
            "cuts": [[0, 1, 2], [1, 2, 4, 5], [2, 5, 8], [0, 2, 7, 8]],
        },
    },
]

# ALGO-FL-032: all_st_mincuts. R-igraph exposes
# `all_st_mincuts(graph, source, target, capacity = NULL)` returning a
# list of `igraph.mincut`-style objects with `$partition1` (source side)
# and `$cut` (edge ids). The set of minimum (s,t) cuts is unique, so the
# Rust runner compares partitions+cuts as a set and also checks `value`.
# All expected vectors are stored 0-based (R reports 1-based).
ALL_ST_MINCUTS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "all_st_mincuts_r_reverse",
        "origin": "rigraph all_st_mincuts on the reversed 4-vertex graph "
        "make_graph(c(2,1,3,1,3,2,4,3), directed=TRUE) with source=3 "
        "target=1 ⇒ 2 minimum cuts of value 2. Stored 0-based: "
        "edges (1,0)(2,0)(2,1)(3,2), source=2 target=0.",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(1, 0), (2, 0), (2, 1), (3, 2)], directed=True
        ),
        "algo": "all_st_mincuts",
        "params": {"source": 2, "target": 0},
        "expected": {
            "value": 2.0,
            "partition1s": [[1, 2], [2]],
            "cuts": [[0, 1], [1, 2]],
        },
    },
    {
        "case": "all_st_mincuts_r_diamond_to_sink",
        "origin": "rigraph all_st_mincuts on the diamond "
        "make_graph(c(1,2,1,3,2,4,3,4,4,5), directed=TRUE) with "
        "source=1 target=5 ⇒ a single minimum cut of value 1 (the "
        "bottleneck edge into the sink). Stored 0-based: edges "
        "(0,1)(0,2)(1,3)(2,3)(3,4), source=0 target=4.",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4)],
            directed=True,
        ),
        "algo": "all_st_mincuts",
        "params": {"source": 0, "target": 4},
        "expected": {
            "value": 1.0,
            "partition1s": [[0, 1, 2, 3]],
            "cuts": [[4]],
        },
    },
    {
        "case": "all_st_mincuts_r_path5",
        "origin": "rigraph all_st_mincuts on the directed path "
        "make_graph(c(1,2,2,3,3,4,4,5), directed=TRUE) with source=1 "
        "target=5 ⇒ 4 minimum cuts of value 1, one per edge. Stored "
        "0-based: edges (0,1)(1,2)(2,3)(3,4), source=0 target=4.",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4)],
            directed=True,
        ),
        "algo": "all_st_mincuts",
        "params": {"source": 0, "target": 4},
        "expected": {
            "value": 1.0,
            "partition1s": [[0], [0, 1], [0, 1, 2], [0, 1, 2, 3]],
            "cuts": [[0], [1], [2], [3]],
        },
    },
]

# ALGO-CN-031: minimum_size_separators (Kanevsky 1993). R-igraph
# exposes `minimum_size_separators(graph)` returning a list of vertex
# sets of minimum size whose removal disconnects the graph. The
# collection is unique and complete, compared as a canonical set.
# Expected values verified against python-igraph 0.11.9 (igraph C
# 0.10.16). Vertex ids are 0-based here; R reports them 1-based.
MINIMUM_SIZE_SEPARATORS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "minimum_size_separators_r_bowtie",
        "origin": "rigraph make_graph('bull')-style bowtie: two "
        "triangles {0,1,2} and {2,3,4} sharing vertex 2. Connectivity "
        "1; the unique minimum separator is the shared vertex {2}.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (0, 2), (1, 2), (2, 3), (2, 4), (3, 4)],
            directed=False,
        ),
        "algo": "minimum_size_separators",
        "params": {},
        "expected": {"separators": [[2]]},
    },
    {
        "case": "minimum_size_separators_r_grid3x3",
        "origin": "rigraph make_lattice(c(3,3)) — 3x3 grid. "
        "Connectivity 2; the minimum separators are the four "
        "corner-cutting pairs around the centre {1,3},{1,5},{3,7},{5,7}.",
        "graph_factory": lambda: ig.Graph.Lattice([3, 3], circular=False),
        "algo": "minimum_size_separators",
        "params": {},
        "expected": {
            "separators": [[1, 3], [1, 5], [3, 7], [5, 7]],
        },
    },
    {
        "case": "minimum_size_separators_r_k23_side01",
        "origin": "rigraph K_{2,3}-style graph: 5 vertices, edges "
        "(2,0)(3,0)(4,0)(2,1)(3,1)(4,1). Connectivity 2; the unique "
        "minimum separator is the degree-3 side {0,1}.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(2, 0), (3, 0), (4, 0), (2, 1), (3, 1), (4, 1)],
            directed=False,
        ),
        "algo": "minimum_size_separators",
        "params": {},
        "expected": {"separators": [[0, 1]]},
    },
]


# ALGO-CN-032: cohesive_blocks (Moody-White 2003). R-igraph exposes
# `cohesive_blocks(graph)` returning a `cohesiveBlocks` object; the
# documented examples in `R/cohesive.blocks.R` use the Moody-White paper
# graph and the science-camp social network. The block enumeration order
# is implementation-defined, so the payload is compared as a canonical
# set of (sorted block, cohesion) pairs. The expected value is computed
# here by the oracle itself, so it is authentic by construction. Vertex
# ids are 0-based here; R reports them 1-based.
def _cohesive_blocks_expected(g: "ig.Graph") -> Dict[str, Any]:
    cb = g.cohesive_blocks()
    pairs = sorted(
        (sorted(int(v) for v in block), int(c))
        for block, c in zip(list(cb), cb.cohesions())
    )
    return {
        "blocks": [p[0] for p in pairs],
        "cohesion": [p[1] for p in pairs],
    }


_CB_MOODY_WHITE = ig.Graph(
    23,
    [
        (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (1, 2), (1, 3), (1, 4),
        (1, 6), (2, 3), (2, 5), (2, 6), (3, 4), (3, 5), (3, 6), (4, 5),
        (4, 6), (4, 20), (5, 6), (6, 7), (6, 10), (6, 13), (6, 18),
        (7, 8), (7, 10), (7, 13), (8, 9), (9, 11), (9, 12), (10, 11),
        (10, 13), (11, 15), (12, 15), (13, 14), (14, 15), (16, 17),
        (16, 18), (16, 19), (17, 19), (17, 20), (18, 19), (18, 21),
        (18, 22), (19, 20), (20, 21), (20, 22), (21, 22),
    ],
    directed=False,
)
_CB_SCIENCE_CAMP = ig.Graph(
    18,
    [
        (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (1, 16), (1, 17), (2, 3),
        (3, 17), (4, 5), (4, 6), (4, 7), (4, 8), (5, 6), (5, 7), (6, 7),
        (6, 8), (7, 8), (7, 16), (8, 9), (8, 10), (9, 11), (9, 12),
        (9, 13), (9, 14), (10, 11), (10, 12), (10, 13), (11, 14),
        (12, 13), (12, 14), (12, 15), (15, 16), (15, 17), (16, 17),
    ],
    directed=False,
)
COHESIVE_BLOCKS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "cohesive_blocks_r_moody_white",
        "origin": "R/cohesive.blocks.R @examples — the Moody-White (2003) "
        "paper graph (23 vertices). Five blocks with cohesions "
        "{1,2,2,3,5}.",
        "graph_factory": lambda: _CB_MOODY_WHITE.copy(),
        "algo": "cohesive_blocks",
        "params": {},
        "expected": _cohesive_blocks_expected(_CB_MOODY_WHITE),
    },
    {
        "case": "cohesive_blocks_r_science_camp",
        "origin": "R/cohesive.blocks.R @examples — the science-camp social "
        "network (18 vertices). Four blocks with cohesions {2,3,3,3}.",
        "graph_factory": lambda: _CB_SCIENCE_CAMP.copy(),
        "algo": "cohesive_blocks",
        "params": {},
        "expected": _cohesive_blocks_expected(_CB_SCIENCE_CAMP),
    },
    {
        "case": "cohesive_blocks_r_bull",
        "origin": "make_graph('bull') — triangle {0,1,2} with horns 0-3 "
        "and 1-4. The triangle is a cohesion-2 block inside the whole "
        "cohesion-1 graph.",
        "graph_factory": lambda: ig.Graph(
            n=5, edges=[(0, 1), (0, 2), (1, 2), (0, 3), (1, 4)],
            directed=False,
        ),
        "algo": "cohesive_blocks",
        "params": {},
        "expected": _cohesive_blocks_expected(
            ig.Graph(
                n=5, edges=[(0, 1), (0, 2), (1, 2), (0, 3), (1, 4)],
                directed=False,
            )
        ),
    },
]

# ALGO-FL-020: gomory_hu_tree. R-igraph exposes
# `gomory_hu_tree(graph, capacity = NULL)` which returns the cut tree
# as a Graph with edge attribute "flow" carrying min-cut weights.
# Tree shape is non-unique (Gusfield depends on scan order); we pin
# only shape invariants here and let the Rust runner verify the
# Gomory-Hu property via `max_flow_value` on every pair.
GOMORY_HU_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "gomory_hu_r_full_5v_unit_caps",
        "origin": "rigraph make_full_graph(5, directed=FALSE) — K_5 "
        "unit caps. Each pair has max-flow 4 (degree); every tree "
        "edge weight = 4.",
        "graph_factory": lambda: ig.Graph.Full(5, directed=False, loops=False),
        "algo": "gomory_hu_tree",
        "params": {"capacity": None},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "flows_len": 4,
            "flows_min": 4.0,
            "is_directed": False,
        },
    },
    {
        "case": "gomory_hu_r_petersen_unit_caps",
        "origin": "rigraph make_graph('Petersen') — 10-vertex 3-regular "
        "unit caps. Petersen graph is 3-edge-connected, so every tree "
        "edge weight = 3.",
        "graph_factory": lambda: ig.Graph.Famous("Petersen"),
        "algo": "gomory_hu_tree",
        "params": {"capacity": None},
        "expected": {
            "vcount": 10,
            "ecount": 9,
            "flows_len": 9,
            "flows_min": 3.0,
            "is_directed": False,
        },
    },
    {
        "case": "gomory_hu_r_path6_unit_caps",
        "origin": "rigraph make_ring(6, directed=FALSE, circular=FALSE) "
        "— P_6 path unit caps. Every cut on a path equals 1; the GH "
        "tree must carry flow 1 on every edge.",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)],
            directed=False,
        ),
        "algo": "gomory_hu_tree",
        "params": {"capacity": None},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "flows_len": 5,
            "flows_min": 1.0,
            "is_directed": False,
        },
    },
    {
        "case": "gomory_hu_r_directed_rejects",
        "origin": "rigraph gomory_hu_tree on a directed graph errors "
        "with 'only defined for undirected graphs' (mirrors C "
        "IGRAPH_EINVAL).",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (1, 2), (2, 3), (3, 0)],
            directed=True,
        ),
        "algo": "gomory_hu_tree",
        "params": {"capacity": None},
        "expected": {"raises": True},
    },
]


# ALGO-FL-030: dominator_tree. R-igraph exposes
# `dominator_tree(graph, root, mode = c("out", "in"))` (and the
# low-level `dominator_tree_impl()`). The R high-level binding returns
# a list `$dom`, `$domtree`, `$leftout` with 1-based vertex ids and
# `-1` at the root. The low-level `_impl` returns the C array as-is
# (0-based, -2 for unreachable). We keep all expected vectors in the
# Rust port's 0-based -1/-2 convention since the conformance runner
# is implementation-agnostic.
DOMINATOR_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dominator_r_5v_tree_out",
        "origin": "rigraph tests/testthat/test-flow.R:228-238 — "
        "6-vertex DAG with edges (1,2),(2,3),(3,4),(2,5),(5,6); "
        "dominator_tree(g, root=1) ⇒ dom[2..6] = 1,2,3,2,5. "
        "Converted to 0-based: edges (0,1),(1,2),(2,3),(1,4),(4,5); "
        "root=0; idom = [-1, 0, 1, 2, 1, 4].",
        "graph_factory": lambda: ig.Graph(
            n=6,
            edges=[(0, 1), (1, 2), (2, 3), (1, 4), (4, 5)],
            directed=True,
        ),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "out"},
        "expected": {
            "idom": [-1, 0, 1, 2, 1, 4],
            "leftout": [],
        },
    },
    {
        "case": "dominator_r_single_vertex_out",
        "origin": "rigraph tests/testthat/test-flow.R:240-243 — "
        "make_empty_graph(n=1, directed=TRUE); dom_tree_one$dom[1] = -1.",
        "graph_factory": lambda: ig.Graph(n=1, edges=[], directed=True),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "out"},
        "expected": {
            "idom": [-1],
            "leftout": [],
        },
    },
    {
        "case": "dominator_r_3v_path_in_unreachable",
        "origin": "rigraph tests/testthat/test-aaa-auto.R:8095-8121 + "
        "_snaps/aaa-auto.md:6862-6873 — path_graph_impl(n=3, "
        "directed=TRUE) with root=1 and mode='in' has only the root "
        "reachable on reverse edges; dom = [-1, -2, -2], leftout = [1, 2].",
        "graph_factory": lambda: ig.Graph(
            n=3, edges=[(0, 1), (1, 2)], directed=True
        ),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "in"},
        "expected": {
            "idom": [-1, -2, -2],
            "leftout": [1, 2],
        },
    },
    {
        "case": "dominator_r_undirected_rejects",
        "origin": "rigraph dominator_tree on an undirected graph errors "
        "(igraph C IGRAPH_EINVAL).",
        "graph_factory": lambda: ig.Graph(
            n=4, edges=[(0, 1), (1, 2), (2, 3)], directed=False
        ),
        "algo": "dominator_tree",
        "params": {"root": 0, "mode": "out"},
        "expected": {"raises": True},
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

# ALGO-GN-030: bipartite_game_gnp / bipartite_game_gnm. Mirrors
# rigraph's `sample_bipartite(n1, n2, type=c('gnp','gnm'), p=NULL,
# m=NULL, directed=FALSE, mode=c('all','out','in'))` (auto-bound
# `sample_bipartite_impl`). RNG state is not portable, so we encode
# structural invariants only.
BIPARTITE_GAME_GNP_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bipartite_gnp_r_undirected_n15_n10_p02_all",
        "origin": "constructed (mirrors sample_bipartite(15, 10, "
        "type='gnp', p=0.2, directed=FALSE, mode='all')): sparse "
        "undirected case",
        "algo": "bipartite_game_gnp",
        "params": {
            "n1": 15,
            "n2": 10,
            "p": 0.2,
            "directed": False,
            "mode": "all",
            "seed": 9_990_101,
        },
        "expected": {
            "vcount": 25,
            "n1": 15,
            "n2": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 8,
            "ecount_max": 80,
            "bipartite_partitions": True,
        },
    },
    {
        "case": "bipartite_gnp_r_directed_n6_n6_p03_all",
        "origin": "constructed (mirrors sample_bipartite(6, 6, "
        "type='gnp', p=0.3, directed=TRUE, mode='all')): directed "
        "with mutual arcs allowed",
        "algo": "bipartite_game_gnp",
        "params": {
            "n1": 6,
            "n2": 6,
            "p": 0.3,
            "directed": True,
            "mode": "all",
            "seed": 9_990_102,
        },
        "expected": {
            "vcount": 12,
            "n1": 6,
            "n2": 6,
            "directed": True,
            "is_simple": True,
            "ecount_min": 5,
            "ecount_max": 72,
            "bipartite_partitions": True,
        },
    },
    {
        "case": "bipartite_gnp_r_undirected_n3_n3_p0_empty",
        "origin": "constructed (mirrors sample_bipartite boundary p=0): "
        "no edges; only the n1+n2 vertices and types partition",
        "algo": "bipartite_game_gnp",
        "params": {
            "n1": 3,
            "n2": 3,
            "p": 0.0,
            "directed": False,
            "mode": "all",
            "seed": 9_990_103,
        },
        "expected": {
            "vcount": 6,
            "n1": 3,
            "n2": 3,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
            "bipartite_partitions": True,
        },
    },
]

BIPARTITE_GAME_GNM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "bipartite_gnm_r_undirected_n12_n10_m40_all",
        "origin": "constructed (mirrors sample_bipartite(12, 10, "
        "type='gnm', m=40, directed=FALSE, mode='all')): exact-count "
        "undirected case",
        "algo": "bipartite_game_gnm",
        "params": {
            "n1": 12,
            "n2": 10,
            "m": 40,
            "directed": False,
            "mode": "all",
            "seed": 9_990_201,
        },
        "expected": {
            "vcount": 22,
            "n1": 12,
            "n2": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 40,
            "ecount_max": 40,
            "bipartite_partitions": True,
        },
    },
    {
        "case": "bipartite_gnm_r_directed_n5_n5_m15_out",
        "origin": "constructed (mirrors sample_bipartite(5, 5, "
        "type='gnm', m=15, directed=TRUE, mode='out')): bottom→top "
        "directed arcs only",
        "algo": "bipartite_game_gnm",
        "params": {
            "n1": 5,
            "n2": 5,
            "m": 15,
            "directed": True,
            "mode": "out",
            "seed": 9_990_202,
        },
        "expected": {
            "vcount": 10,
            "n1": 5,
            "n2": 5,
            "directed": True,
            "is_simple": True,
            "ecount_min": 15,
            "ecount_max": 15,
            "bipartite_partitions": True,
            "edges_bottom_to_top": True,
        },
    },
    {
        "case": "bipartite_gnm_r_n4_n3_m12_all_complete",
        "origin": "constructed (mirrors sample_bipartite boundary m=max): "
        "undirected mode='all' yields complete K_{4,3}",
        "algo": "bipartite_game_gnm",
        "params": {
            "n1": 4,
            "n2": 3,
            "m": 12,
            "directed": False,
            "mode": "all",
            "seed": 9_990_203,
        },
        "expected": {
            "vcount": 7,
            "n1": 4,
            "n2": 3,
            "directed": False,
            "is_simple": True,
            "ecount_min": 12,
            "ecount_max": 12,
            "bipartite_partitions": True,
        },
    },
]

# ALGO-GN-031: iea_game. rigraph re-exports the C entry point through
# the auto-bound `iea_game_impl`. RNG state is not portable, so we
# capture structural invariants only: vcount==n, ecount==m EXACT,
# directedness preserved, no self-loops when loops=FALSE.
IEA_GAME_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "iea_r_directed_loops_n35_m140",
        "origin": "constructed (mirrors rigraph iea_game_impl(n=35, m=140, "
        "directed=TRUE, loops=TRUE)): directed multigraph with self-loops",
        "algo": "iea_game",
        "params": {
            "n": 35,
            "m": 140,
            "directed": True,
            "loops": True,
            "seed": 5_553_001,
        },
        "expected": {
            "vcount": 35,
            "ecount": 140,
            "directed": True,
            "no_self_loops": False,
        },
    },
    {
        "case": "iea_r_undirected_no_loops_n18_m45",
        "origin": "constructed (mirrors rigraph iea_game_impl(n=18, m=45, "
        "directed=FALSE, loops=FALSE)): simple-pair undirected multigraph",
        "algo": "iea_game",
        "params": {
            "n": 18,
            "m": 45,
            "directed": False,
            "loops": False,
            "seed": 5_553_002,
        },
        "expected": {
            "vcount": 18,
            "ecount": 45,
            "directed": False,
            "no_self_loops": True,
        },
    },
    {
        "case": "iea_r_directed_loops_n10_m0_empty",
        "origin": "constructed (mirrors rigraph iea_game_impl boundary "
        "m=0): vcount preserved, edgeless graph",
        "algo": "iea_game",
        "params": {
            "n": 10,
            "m": 0,
            "directed": True,
            "loops": True,
            "seed": 5_553_003,
        },
        "expected": {
            "vcount": 10,
            "ecount": 0,
            "directed": True,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-014: preference_game. Mirrors rigraph's `sample_pref(nodes,
# types, type.dist, fixed.sizes, pref.matrix, ...)` — the auto-bound
# `preference_game_impl`. RNG state is not portable, so we encode
# structural invariants only.
PREFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "preference_r_n5_2types_uniform",
        "origin": "tests/testthat/test-aaa-auto.R::preference_game_impl basic — "
        "nodes=5, types=2, type_dist=c(0.5,0.5), pref_matrix all 0.5",
        "algo": "preference_game",
        "params": {
            "nodes": 5,
            "types": 2,
            "type_dist": [0.5, 0.5],
            "fixed_sizes": False,
            "pref_matrix": [
                [0.5, 0.5],
                [0.5, 0.5],
            ],
            "directed": False,
            "loops": False,
            "seed": 1_110_001,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "is_simple": True,
            # 5 vertices, undirected, max 10 edges; expect roughly half.
            "ecount_min": 0,
            "ecount_max": 10,
            "diagonal_only_pref": False,
            "max_type": 1,
        },
    },
    {
        "case": "preference_r_fixed_sizes_balanced_diag",
        "origin": "constructed (mirrors sample_pref): fixed_sizes=TRUE "
        "evenly splits nodes; diagonal pref keeps edges in-block",
        "algo": "preference_game",
        "params": {
            "nodes": 24,
            "types": 4,
            "type_dist": None,
            "fixed_sizes": True,
            "pref_matrix": [
                [0.5, 0.0, 0.0, 0.0],
                [0.0, 0.5, 0.0, 0.0],
                [0.0, 0.0, 0.5, 0.0],
                [0.0, 0.0, 0.0, 0.5],
            ],
            "directed": False,
            "loops": False,
            "seed": 1_110_002,
        },
        "expected": {
            "vcount": 24,
            "directed": False,
            "is_simple": True,
            # 4 blocks of 6, each C(6,2)*0.5 = 7.5; total ≈ 30.
            "ecount_min": 12,
            "ecount_max": 60,
            "diagonal_only_pref": True,
            "max_type": 3,
        },
    },
    {
        "case": "preference_r_directed_full_pref",
        "origin": "constructed (mirrors sample_pref): directed graph, "
        "uniform pref 0.3 across all type pairs",
        "algo": "preference_game",
        "params": {
            "nodes": 30,
            "types": 3,
            "type_dist": [1.0, 1.0, 1.0],
            "fixed_sizes": False,
            "pref_matrix": [
                [0.3, 0.3, 0.3],
                [0.3, 0.3, 0.3],
                [0.3, 0.3, 0.3],
            ],
            "directed": True,
            "loops": False,
            "seed": 1_110_003,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "is_simple": True,
            # Directed, no loops, max 30*29 = 870 edges; E ≈ 0.3 * 870 = 261.
            "ecount_min": 180,
            "ecount_max": 360,
            "diagonal_only_pref": False,
            "max_type": 2,
        },
    },
]

# ALGO-GN-014: asymmetric_preference_game. Mirrors rigraph's
# `sample_asym_pref(nodes, types, type.dist.matrix, pref.matrix, ...)`
# (auto-bound `asymmetric_preference_game_impl`).
ASYMMETRIC_PREFERENCE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "asym_preference_r_n5_2x2_uniform",
        "origin": "tests/testthat/test-aaa-auto.R::asymmetric_preference_game_impl "
        "basic — nodes=5, 2x2 type_dist_matrix and pref_matrix all 0.5",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 5,
            "no_out_types": 2,
            "no_in_types": 2,
            "type_dist_matrix": [
                [0.5, 0.5],
                [0.5, 0.5],
            ],
            "pref_matrix": [
                [0.5, 0.5],
                [0.5, 0.5],
            ],
            "loops": False,
            "seed": 1_111_001,
        },
        "expected": {
            "vcount": 5,
            "directed": True,
            "is_simple": True,
            # 5 vertices, no loops ⇒ 5*4 = 20 directed slot ceiling.
            "ecount_min": 0,
            "ecount_max": 20,
            "max_out_type": 1,
            "max_in_type": 1,
        },
    },
    {
        "case": "asym_preference_r_block_balanced_p06",
        "origin": "constructed (mirrors sample_asym_pref): joint dist "
        "diagonal pins out_type==in_type for every vertex; pref diagonal "
        "p=0.6",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 30,
            "no_out_types": 3,
            "no_in_types": 3,
            "type_dist_matrix": [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            "pref_matrix": [
                [0.6, 0.0, 0.0],
                [0.0, 0.6, 0.0],
                [0.0, 0.0, 0.6],
            ],
            "loops": True,
            "seed": 1_111_002,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "is_simple": False,
            # 3 blocks of 10 each; each block: 10*10=100 slots at p=0.6
            # => 60 edges (incl. self-loops). Total ≈ 180; allow band.
            "ecount_min": 100,
            "ecount_max": 240,
            "max_out_type": 2,
            "max_in_type": 2,
        },
    },
    {
        "case": "asym_preference_r_zero_pref_edgeless",
        "origin": "constructed (mirrors sample_asym_pref with "
        "pref_matrix all zero): vcount preserved, edgeless",
        "algo": "asymmetric_preference_game",
        "params": {
            "nodes": 20,
            "no_out_types": 2,
            "no_in_types": 2,
            "type_dist_matrix": None,
            "pref_matrix": [
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            "loops": False,
            "seed": 1_111_003,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
            "max_out_type": 1,
            "max_in_type": 1,
        },
    },
]

# ALGO-GN-015: establishment_game. Mirrors rigraph's
# `sample_traits(nodes, types, k, type.dist, pref.matrix, directed)`
# — a thin R wrapper on `igraph_establishment_game`. RNG state is not
# portable, so we encode structural invariants only.
ESTABLISHMENT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "establishment_r_uniform_p05_n50_2types_k3",
        "origin": "tests/testthat/test-aaa-auto.R::sample_traits "
        "basic — sample_traits(nodes=50, types=2, k=3, "
        "pref.matrix=full(0.5))",
        "algo": "establishment_game",
        "params": {
            "nodes": 50,
            "types": 2,
            "k": 3,
            "type_dist": [1.0, 1.0],
            "pref_matrix": [
                [0.5, 0.5],
                [0.5, 0.5],
            ],
            "directed": False,
            "seed": 2_222_001,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            "is_simple": True,
            # E[edges] ≈ (n-k)*k * 0.5 = 47*3*0.5 = 70.5
            "ecount_min": 35,
            "ecount_max": 110,
            "max_type": 1,
        },
    },
    {
        "case": "establishment_r_directed_full_pref_n40_3types_k4",
        "origin": "constructed (mirrors sample_traits with directed=TRUE "
        "and an asymmetric pref.matrix all = 1)",
        "algo": "establishment_game",
        "params": {
            "nodes": 40,
            "types": 3,
            "k": 4,
            "type_dist": None,
            "pref_matrix": [
                [1.0, 0.5, 0.2],
                [0.5, 1.0, 0.5],
                [0.2, 0.5, 1.0],
            ],
            "directed": True,
            "seed": 2_222_002,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "is_simple": True,
            # E[edges] ≈ (40-4)*4 * mean(pref) = 144 * (5.4/9) = 86.4
            "ecount_min": 50,
            "ecount_max": 144,
            "max_type": 2,
        },
    },
    {
        "case": "establishment_r_full_p1_three_types_k1",
        "origin": "constructed (mirrors sample_traits with k=1 and "
        "pref.matrix=ones): exactly (n-k) edges",
        "algo": "establishment_game",
        "params": {
            "nodes": 25,
            "types": 3,
            "k": 1,
            "type_dist": None,
            "pref_matrix": [
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            "directed": False,
            "seed": 2_222_003,
        },
        "expected": {
            "vcount": 25,
            "directed": False,
            "is_simple": True,
            "ecount_min": 24,  # (n-k)*k = 24
            "ecount_max": 24,
            "max_type": 2,
        },
    },
]

# ALGO-GN-016: callaway_traits_game. Mirrors rigraph's
# `sample_traits_callaway(nodes, types, edges.per.step, type.dist,
# pref.matrix, directed)` — a thin R wrapper on
# `igraph_callaway_traits_game`. Differs from sample_traits in that BOTH
# vertices of each candidate edge are drawn uniformly from the existing
# population [0, i] (inclusive), so self-loops and multi-edges are
# allowed by construction. RNG state is not portable, so we encode
# structural invariants only.
CALLAWAY_TRAITS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "callaway_r_uniform_p05_n50_2types_eps3",
        "origin": "tests/testthat/test-aaa-auto.R::sample_traits_callaway "
        "basic — sample_traits_callaway(nodes=50, types=2, "
        "edges.per.step=3, pref.matrix=full(0.5))",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 50,
            "types": 2,
            "edges_per_step": 3,
            "type_dist": [1.0, 1.0],
            "pref_matrix": [
                [0.5, 0.5],
                [0.5, 0.5],
            ],
            "directed": False,
            "seed": 2_223_001,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            # E[edges] ≈ (n-1)*eps*0.5 = 49*3*0.5 = 73.5; max = 147
            "ecount_min": 35,
            "ecount_max": 147,
            "max_type": 1,
        },
    },
    {
        "case": "callaway_r_directed_full_pref_n40_3types_eps2",
        "origin": "constructed (mirrors sample_traits_callaway with "
        "directed=TRUE and an asymmetric pref.matrix)",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 40,
            "types": 3,
            "edges_per_step": 2,
            "type_dist": None,
            "pref_matrix": [
                [1.0, 0.5, 0.2],
                [0.5, 1.0, 0.5],
                [0.2, 0.5, 1.0],
            ],
            "directed": True,
            "seed": 2_223_002,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            # E[edges] ≈ (n-1)*eps * mean(pref) = 78 * (5.4/9) = 46.8;
            # max = 78
            "ecount_min": 20,
            "ecount_max": 78,
            "max_type": 2,
        },
    },
    {
        "case": "callaway_r_full_p1_three_types_eps1",
        "origin": "constructed (mirrors sample_traits_callaway with "
        "edges.per.step=1 and pref.matrix=ones): exactly (n-1) edges",
        "algo": "callaway_traits_game",
        "params": {
            "nodes": 25,
            "types": 3,
            "edges_per_step": 1,
            "type_dist": None,
            "pref_matrix": [
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            "directed": False,
            "seed": 2_223_003,
        },
        "expected": {
            "vcount": 25,
            "directed": False,
            "ecount_min": 24,  # (n-1)*eps = 24
            "ecount_max": 24,
            "max_type": 2,
        },
    },
]

# ALGO-GN-017: cited_type_game. Mirrors rigraph's
# `sample_cit_types(types, pref, edges, directed=TRUE)` — an R wrapper
# on `igraph_cited_type_game`. Vertex types are PRE-ASSIGNED by the
# caller (not sampled), and each new vertex i ∈ [1, nodes) adds
# `edges` outgoing citations to previously-added vertices with
# probability ∝ pref[type[v]]. Multi-edges allowed; self-loops only
# via sum=0 fallback. RNG state is not portable — structural invariants
# only.
CITED_TYPE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "cited_type_r_uniform_pref_n30_2types_eps3",
        "origin": "tests/testthat/test-aaa-auto.R::sample_cit_types — "
        "sample_cit_types(types=[0,1,0,1,...], pref=[1,1], edges=3, "
        "directed=TRUE)",
        "algo": "cited_type_game",
        "params": {
            "nodes": 30,
            "types": [v % 2 for v in range(30)],
            "pref": [1.0, 1.0],
            "edges_per_step": 3,
            "directed": True,
            "seed": 2_224_001,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "ecount_min": 87,  # (30-1)*3 = 87
            "ecount_max": 87,
            "no_self_loops": True,
            "max_type": 1,
        },
    },
    {
        "case": "cited_type_r_concentrated_pref_n50_3types_eps2_undirected",
        "origin": "constructed (sample_cit_types with concentrated "
        "pref=[10, 1, 0.05]): heavy citation skew, no self-loops, "
        "exact (n-1)*eps = 98 edges",
        "algo": "cited_type_game",
        "params": {
            "nodes": 50,
            "types": [v % 3 for v in range(50)],
            "pref": [10.0, 1.0, 0.05],
            "edges_per_step": 2,
            "directed": False,
            "seed": 2_224_002,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            "ecount_min": 98,
            "ecount_max": 98,
            "no_self_loops": True,
            "max_type": 2,
        },
    },
    {
        "case": "cited_type_r_eps1_single_type_n15",
        "origin": "constructed (sample_cit_types with edges=1, pref=[1]): "
        "exactly (n-1) edges = 14, no self-loops",
        "algo": "cited_type_game",
        "params": {
            "nodes": 15,
            "types": [0 for _ in range(15)],
            "pref": [1.0],
            "edges_per_step": 1,
            "directed": True,
            "seed": 2_224_003,
        },
        "expected": {
            "vcount": 15,
            "directed": True,
            "ecount_min": 14,
            "ecount_max": 14,
            "no_self_loops": True,
            "max_type": 0,
        },
    },
]

# ALGO-GN-029: citing_cited_type_game. Mirrors rigraph's
# `sample_cit_cit_types(types, pref, edges, directed=TRUE)` — an R
# wrapper on `igraph_citing_cited_type_game`. Generalises cited_type by
# also conditioning on the citing vertex's type: weight is
# pref[type[citing]][type[cited]] (one psumtree per citing type).
# Multi-edges allowed when eps≥2; NEVER self-loops (uniform fallback
# samples [0, i)). RNG state is not portable — structural invariants only.
CITING_CITED_TYPE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "citing_cited_r_uniform_pref_n30_2types_eps3_directed",
        "origin": "tests/testthat/test-aaa-auto.R::sample_cit_cit_types — "
        "sample_cit_cit_types(types=[0,1,...], pref=2x2 ones, edges=3, "
        "directed=TRUE)",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 30,
            "types": [v % 2 for v in range(30)],
            "pref": [[1.0, 1.0], [1.0, 1.0]],
            "edges_per_step": 3,
            "directed": True,
            "seed": 3_224_001,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "ecount_min": 87,  # (30-1)*3 = 87
            "ecount_max": 87,
            "no_self_loops": True,
            "max_type": 1,
        },
    },
    {
        "case": "citing_cited_r_concentrated_pref_n50_3types_eps2_undirected",
        "origin": "constructed (sample_cit_cit_types with concentrated "
        "3x3 diagonal pref): same-type citing concentration, no "
        "self-loops, exact (n-1)*eps = 98 edges",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 50,
            "types": [v % 3 for v in range(50)],
            "pref": [
                [10.0, 0.05, 0.05],
                [0.05, 10.0, 0.05],
                [0.05, 0.05, 10.0],
            ],
            "edges_per_step": 2,
            "directed": False,
            "seed": 3_224_002,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            "ecount_min": 98,
            "ecount_max": 98,
            "no_self_loops": True,
            "max_type": 2,
        },
    },
    {
        "case": "citing_cited_r_eps1_single_type_n15_directed",
        "origin": "constructed (sample_cit_cit_types with edges=1, "
        "pref=[[1]]): exactly (n-1) edges = 14, no self-loops",
        "algo": "citing_cited_type_game",
        "params": {
            "nodes": 15,
            "types": [0 for _ in range(15)],
            "pref": [[1.0]],
            "edges_per_step": 1,
            "directed": True,
            "seed": 3_224_003,
        },
        "expected": {
            "vcount": 15,
            "directed": True,
            "ecount_min": 14,
            "ecount_max": 14,
            "no_self_loops": True,
            "max_type": 0,
        },
    },
]

# ALGO-GN-018: lastcit_game. Mirrors rigraph's `sample_last_cit`
# (R wrapper on `igraph_lastcit_game`). Each new vertex emits
# `edges_per_node` outgoing citations; cited vertices' weights decay
# with the time since their last citation, binned into `agebins`
# buckets. The psumtree implementation gives O(log n) update + search.
# Never self-loops by construction; may produce multi-edges when
# edges_per_node ≥ 2.
LASTCIT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lastcit_r_uniform_n30_3bins_eps3",
        "origin": "constructed (mirrors igraph::sample_last_cit(n=30, "
        "edges=3, agebins=3, pref=[1,1,1,1], directed=TRUE)) — "
        "uniform-pref baseline; ecount = (n-1)*eps = 87",
        "algo": "lastcit_game",
        "params": {
            "nodes": 30,
            "edges_per_node": 3,
            "agebins": 3,
            "preference": [1.0, 1.0, 1.0, 1.0],
            "directed": True,
            "seed": 9_995_001,
        },
        "expected": {
            "vcount": 30,
            "directed": True,
            "ecount_min": 87,
            "ecount_max": 87,
            "no_self_loops": True,
        },
    },
    {
        "case": "lastcit_r_steep_decay_n45_5bins_eps2_undirected",
        "origin": "constructed (mirrors igraph::sample_last_cit(n=45, "
        "edges=2, agebins=5, pref=[16,8,4,2,1,0.5], directed=FALSE)) — "
        "geometric-decay preference across 5 age bins; tests the age "
        "sweep at multiple bin boundaries",
        "algo": "lastcit_game",
        "params": {
            "nodes": 45,
            "edges_per_node": 2,
            "agebins": 5,
            "preference": [16.0, 8.0, 4.0, 2.0, 1.0, 0.5],
            "directed": False,
            "seed": 9_995_002,
        },
        "expected": {
            "vcount": 45,
            "directed": False,
            "ecount_min": 88,
            "ecount_max": 88,
            "no_self_loops": True,
        },
    },
    {
        "case": "lastcit_r_eps1_small_n15",
        "origin": "constructed (mirrors igraph::sample_last_cit(n=15, "
        "edges=1, agebins=2, pref=[3,1,1], directed=TRUE)) — "
        "minimal eps=1 case; ecount = (n-1) = 14, every step emits "
        "exactly one citation",
        "algo": "lastcit_game",
        "params": {
            "nodes": 15,
            "edges_per_node": 1,
            "agebins": 2,
            "preference": [3.0, 1.0, 1.0],
            "directed": True,
            "seed": 9_995_003,
        },
        "expected": {
            "vcount": 15,
            "directed": True,
            "ecount_min": 14,
            "ecount_max": 14,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-019: recent_degree_game. Mirrors rigraph's
# `sample_recent_degree(n, power, window, m, outpref, zero.appeal,
# directed)` (R wrapper on `igraph_recent_degree_game`). Each step
# draws m citations weighted by `pow(recent_in_degree, power) +
# zero_appeal`; edges added at step `i - window` are expired from the
# BIT-tree. Never self-loops by construction.
RECENT_DEGREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "recent_degree_r_pow075_window4_m2_directed",
        "origin": "constructed (mirrors igraph::sample_recent_degree(n=35, "
        "power=0.75, window=4, m=2, outpref=FALSE, zero.appeal=1.0, "
        "directed=TRUE)) — sub-linear preferential attachment with a "
        "short 4-step memory window; ecount = (35-1)*2 = 68",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 35,
            "power": 0.75,
            "time_window": 4,
            "m": 2,
            "outpref": False,
            "zero_appeal": 1.0,
            "directed": True,
            "seed": 9_997_001,
        },
        "expected": {
            "vcount": 35,
            "directed": True,
            "ecount_min": 68,
            "ecount_max": 68,
            "no_self_loops": True,
        },
    },
    {
        "case": "recent_degree_r_outpref_pow1_window6_m3_undirected",
        "origin": "constructed (mirrors igraph::sample_recent_degree(n=20, "
        "power=1.0, window=6, m=3, outpref=TRUE, zero.appeal=0.5, "
        "directed=FALSE)) — outpref=TRUE makes the source vertex's "
        "outgoing degree also feed the BIT-tree (in undirected mode this "
        "is the natural choice); exercises both update branches",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 20,
            "power": 1.0,
            "time_window": 6,
            "m": 3,
            "outpref": True,
            "zero_appeal": 0.5,
            "directed": False,
            "seed": 9_997_002,
        },
        "expected": {
            "vcount": 20,
            "directed": False,
            "ecount_min": 57,  # (20-1)*3 = 57
            "ecount_max": 57,
            "no_self_loops": True,
        },
    },
    {
        "case": "recent_degree_r_minimal_m1_long_window",
        "origin": "constructed (mirrors igraph::sample_recent_degree(n=20, "
        "power=2.0, window=20, m=1, outpref=FALSE, zero.appeal=1.0, "
        "directed=TRUE)) — minimal m=1 case (= recursive tree) with "
        "window=n so no edge ever expires; ecount = n-1 = 19",
        "algo": "recent_degree_game",
        "params": {
            "nodes": 20,
            "power": 2.0,
            "time_window": 20,
            "m": 1,
            "outpref": False,
            "zero_appeal": 1.0,
            "directed": True,
            "seed": 9_997_003,
        },
        "expected": {
            "vcount": 20,
            "directed": True,
            "ecount_min": 19,
            "ecount_max": 19,
            "no_self_loops": True,
        },
    },
]

# ALGO-GN-020: barabasi_game_psumtree / barabasi_game_psumtree_multiple.
# Mirrors rigraph's `sample_pa(n, power, m, out.pref, zero.appeal,
# directed, algorithm="psumtree")` and `algorithm="psumtree.multiple"`
# (R wrapper on `igraph_barabasi_game`). The SIMPLE variant prevents
# within-step multi-edges via per-draw weight zeroing; the MULTIPLE
# variant snapshots the BIT sum once per step and uses the `m >= i`
# early-cite branch. Never self-loops by construction.
BARABASI_PSUMTREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_psumtree_r_classic_directed_m2",
        "origin": "constructed (mirrors igraph::sample_pa(n=40, power=1.0, "
        "m=2, out.pref=FALSE, zero.appeal=1.0, directed=TRUE, "
        "algorithm='psumtree')) — classical BA kernel; ecount = "
        "(40-1)*2 = 78",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 40,
            "power": 1.0,
            "m": 2,
            "outpref": False,
            "a": 1.0,
            "directed": True,
            "variant": "psumtree",
            "seed": 10_000_001,
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
        "case": "barabasi_psumtree_r_multiple_pow15_directed_m3",
        "origin": "constructed (mirrors igraph::sample_pa(n=30, power=1.5, "
        "m=3, out.pref=FALSE, zero.appeal=1.0, directed=TRUE, "
        "algorithm='psumtree.multiple')) — saturation triangle deducts "
        "3 edges from 87",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 30,
            "power": 1.5,
            "m": 3,
            "outpref": False,
            "a": 1.0,
            "directed": True,
            "variant": "psumtree_multiple",
            "seed": 10_000_002,
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
        "case": "barabasi_psumtree_r_undirected_outpref_m2",
        "origin": "constructed (mirrors igraph::sample_pa(n=35, power=1.0, "
        "m=2, out.pref=TRUE, zero.appeal=0.5, directed=FALSE, "
        "algorithm='psumtree')) — undirected forces out.pref=TRUE",
        "algo": "barabasi_game_psumtree",
        "params": {
            "nodes": 35,
            "power": 1.0,
            "m": 2,
            "outpref": True,
            "a": 0.5,
            "directed": False,
            "variant": "psumtree",
            "seed": 10_000_003,
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

# ALGO-GN-021: barabasi_aging_game. rigraph exposes
# `sample_pa_age()` / `sample_last_cit()` family but no direct R wrapper
# (the C kernel is reached through `igraph_barabasi_aging_game`); the
# fixtures here pin the C-kernel invariants. Without `outseq` (or
# `out.seq`), ecount = (nodes - 1) * m exactly. RNG state is not
# portable across implementations.
BARABASI_AGING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "barabasi_aging_r_classic_no_aging_directed_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=40, "
        "m=2, out.pref=FALSE, pa.exp=1.0, aging.exp=0.0, aging.bins=10, "
        "zero.deg.appeal=1.0, zero.age.appeal=1.0, deg.coef=1.0, "
        "age.coef=1.0, directed=TRUE)) — aging.exp=0 collapses age "
        "term to a constant",
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
            "seed": 10_000_101,
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
        "case": "barabasi_aging_r_strong_aging_directed_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=40, "
        "m=2, out.pref=FALSE, pa.exp=1.0, aging.exp=-1.0, aging.bins=10, "
        "zero.deg.appeal=1.0, zero.age.appeal=1.0, deg.coef=1.0, "
        "age.coef=1.0, directed=TRUE)) — aging.exp=-1 favours fresh "
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
            "seed": 10_000_102,
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
        "case": "barabasi_aging_r_outpref_undirected_m2",
        "origin": "constructed (mirrors igraph_barabasi_aging_game(n=35, "
        "m=2, out.pref=TRUE, pa.exp=1.0, aging.exp=-0.5, aging.bins=8, "
        "zero.deg.appeal=0.5, zero.age.appeal=1.0, deg.coef=1.0, "
        "age.coef=1.0, directed=FALSE)) — undirected + outpref",
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
            "seed": 10_000_103,
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

# ALGO-GN-032: recent_degree_aging_game. rigraph exposes
# `sample_last_cit()` style wrapper; the C kernel is
# igraph_recent_degree_aging_game. RNG not portable, so conformance is
# structural: vcount, ecount exact, no_self_loops, directed flag.
RECENT_DEGREE_AGING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "recent_degree_aging_r_no_aging_directed_m2",
        "origin": "constructed (mirrors igraph_recent_degree_aging_game("
        "n=40, m=2, out.pref=FALSE, pa.exp=1.0, aging.exp=0.0, "
        "aging.bins=10, time.window=5, zero.appeal=1.0, directed=TRUE)) "
        "— aging.exp=0 collapses age term; ecount = 78",
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
            "seed": 10_001_101,
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
        "case": "recent_degree_aging_r_strong_aging_directed_m2",
        "origin": "constructed (mirrors igraph_recent_degree_aging_game("
        "n=40, m=2, out.pref=FALSE, pa.exp=1.0, aging.exp=-1.0, "
        "aging.bins=10, time.window=8, zero.appeal=1.0, directed=TRUE)) "
        "— aging.exp=-1 suppresses old vertices; ecount = 78",
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
            "seed": 10_001_102,
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
        "case": "recent_degree_aging_r_outpref_undirected_m2",
        "origin": "constructed (mirrors igraph_recent_degree_aging_game("
        "n=35, m=2, out.pref=TRUE, pa.exp=1.0, aging.exp=-0.5, "
        "aging.bins=8, time.window=10, zero.appeal=0.5, directed=FALSE)) "
        "— undirected + outpref; ecount = 68",
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
            "seed": 10_001_103,
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

# ALGO-GN-022: dot_product_game. rigraph exposes
# `sample_dot_product(vecs, directed = FALSE)` as the canonical R-side
# wrapper (the C kernel is `igraph_dot_product_game`); the autobinding
# matches the C signature exactly so the underlying generator is
# verbatim. RNG state is not portable across implementations, so the
# manifest pins latent-position layouts that clamp every pair to {0, 1}
# (always-/never-edge regimes), giving exact ecount under any RNG. The
# third case exercises both warning regimes for diagnostic coverage.
DOT_PRODUCT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "dot_product_r_all_ones_complete_n8_undirected",
        "origin": "constructed (mirrors sample_dot_product(vecs = "
        "matrix(1.0, nrow=1, ncol=8), directed = FALSE)) — every dot "
        "product is 1.0; with strict gen_unit() < prob (gen_unit ∈ "
        "[0, 1)) every pair fires; ecount = 8·7/2 = 28 exact",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.0]] * 8,
            "directed": False,
            "seed": 10_003_101,
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
        "case": "dot_product_r_orthogonal_groups_n8_undirected",
        "origin": "constructed (mirrors sample_dot_product(vecs = cbind("
        "matrix(c(1,0), nrow=2, ncol=4), matrix(c(0,1), nrow=2, "
        "ncol=4)), directed = FALSE)) — same-group dot = 1 always edge, "
        "cross-group dot = 0 never edge; ecount = 2·C(4,2) = 12 exact",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.0, 0.0]] * 4 + [[0.0, 1.0]] * 4,
            "directed": False,
            "seed": 10_003_102,
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
        "case": "dot_product_r_mixed_clamp_n10_directed",
        "origin": "constructed (mirrors sample_dot_product(vecs = "
        "matrix(c(rep(1.5, 5), rep(-0.5, 5)), nrow=1), directed = "
        "TRUE)) — same-(+) dot = 2.25 always edge (no RNG draw, 5·4 = "
        "20); same-(−) dot = 0.25 Bernoulli (5·4 attempts → 0..20); "
        "cross dot = -0.75 always skip; ecount ∈ [20, 40]; exercises "
        "both clamp warnings",
        "algo": "dot_product_game",
        "params": {
            "vecs": [[1.5]] * 5 + [[-0.5]] * 5,
            "directed": True,
            "seed": 10_003_103,
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

# ALGO-GN-023: correlated_game + correlated_pair_game. rigraph exposes
# `sample_correlated_gnp(old.graph, corr, p, permutation = NULL)` and
# `sample_correlated_gnp_pair(n, corr, p, directed = FALSE,
# permutation = NULL)` as the canonical wrappers around
# `igraph_correlated_game` / `igraph_correlated_pair_game`. RNG state
# is not portable to R's RNGkind; structural-only fixtures pin
# corr = 1 cases (exact copy of old graph) and use 6σ Binomial bands
# for the pair-game ecounts.
CORRELATED_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "correlated_r_corr1_path_n4_exact_copy",
        "origin": "constructed (mirrors sample_correlated_gnp(old.graph = "
        "make_graph(c(1,2, 2,3, 3,4), n=4, directed=FALSE), corr=1.0, "
        "p=0.5, permutation=NULL)) — corr=1 yields exact copy of old; "
        "ecount = 3 exact",
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
            "seed": 11_042_301,
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
        "case": "correlated_r_corr1_cycle_n5_permutation_reverse",
        "origin": "constructed (mirrors sample_correlated_gnp(old.graph = "
        "make_ring(5, directed=FALSE), corr=1.0, p=0.5, "
        "permutation=c(5,4,3,2,1) - 1 in 0-based)) — permutation only "
        "relabels vertices, ecount = 5 exact",
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
            "seed": 11_042_302,
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
        "case": "correlated_pair_r_n30_corr5_p2_undirected",
        "origin": "constructed (mirrors sample_correlated_gnp_pair(n=30, "
        "corr=0.5, p=0.2, directed=FALSE, permutation=NULL)) — both "
        "graphs ER-marginal: mean ecount = C(30,2)·0.2 = 87, σ ≈ 8.34, "
        "conservative band [40, 140]",
        "algo": "correlated_pair_game",
        "params": {
            "n": 30,
            "corr": 0.5,
            "p": 0.2,
            "directed": False,
            "permutation": None,
            "seed": 11_042_311,
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
        "case": "correlated_pair_r_n20_corr8_p25_directed",
        "origin": "constructed (mirrors sample_correlated_gnp_pair(n=20, "
        "corr=0.8, p=0.25, directed=TRUE, permutation=NULL)) — both "
        "graphs ER-marginal: mean ecount = 20·19·0.25 = 95, σ ≈ 8.44, "
        "conservative band [45, 150]",
        "algo": "correlated_pair_game",
        "params": {
            "n": 20,
            "corr": 0.8,
            "p": 0.25,
            "directed": True,
            "permutation": None,
            "seed": 11_042_312,
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

# ALGO-GN-024: degree_sequence_game (CONFIGURATION). rigraph exposes this
# as `sample_degseq(out.deg, in.deg = NULL, method = "configuration")`,
# wrapping the same C entry point. The configuration variant is
# degree-preserving by construction, so the manifest pins vcount, ecount
# and the exact degree sequence — no bands needed.
DEGREE_SEQUENCE_CONFIG_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_config_r_undirected_n12_descending",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=c(4,4,3,3,3,2,2,2,1,1,1,0), method='configuration')`): "
        "decreasing degree sequence, Σd=26 (even).",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [4, 4, 3, 3, 3, 2, 2, 2, 1, 1, 1, 0],
            "in_degrees": None,
            "seed": 8_640_001,
        },
        "expected": {
            "vcount": 12,
            "directed": False,
            "ecount": 13,
            "out_degrees": [4, 4, 3, 3, 3, 2, 2, 2, 1, 1, 1, 0],
            "in_degrees": None,
        },
    },
    {
        "case": "degseq_config_r_directed_n5_balanced",
        "origin": "constructed (mirrors `sample_degseq(out.deg=c(2,1,1,2,2), "
        "in.deg=c(1,2,2,1,2), method='configuration')`): directed multigraph; "
        "Σout=Σin=8.",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [2, 1, 1, 2, 2],
            "in_degrees": [1, 2, 2, 1, 2],
            "seed": 8_640_002,
        },
        "expected": {
            "vcount": 5,
            "directed": True,
            "ecount": 8,
            "out_degrees": [2, 1, 1, 2, 2],
            "in_degrees": [1, 2, 2, 1, 2],
        },
    },
    {
        "case": "degseq_config_r_undirected_all_isolated",
        "origin": "constructed (mirrors `sample_degseq(out.deg=rep(0, 5), "
        "method='configuration')`): five isolated vertices; ecount must be 0.",
        "algo": "degree_sequence_game_configuration",
        "params": {
            "out_degrees": [0, 0, 0, 0, 0],
            "in_degrees": None,
            "seed": 8_640_003,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "ecount": 0,
            "out_degrees": [0, 0, 0, 0, 0],
            "in_degrees": None,
        },
    },
]

# ALGO-GN-026: degree_sequence_game (FAST_HEUR_SIMPLE method). rigraph
# exposes this as `sample_degseq(out.deg, in.deg=NULL,
# method="fast.heur.simple")`, returning a *simple* (no self-loops,
# no multi-edges) graph that exactly realises the given degree sequence.
# RNG state is not portable across implementations, so the manifest pins
# structural invariants only — vcount, ecount, exact (out/in-)degrees,
# simplicity.
DEGREE_SEQUENCE_FAST_HEUR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_fastheur_r_undirected_n10_descending",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=c(4,3,3,3,3,2,2,2,1,1), method='fast.heur.simple')`): "
        "all-positive descending degree sequence, Σd=24.",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [4, 3, 3, 3, 3, 2, 2, 2, 1, 1],
            "in_degrees": None,
            "seed": 8_660_001,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 12,
            "out_degrees": [4, 3, 3, 3, 3, 2, 2, 2, 1, 1],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_fastheur_r_3regular_n10",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=rep(3, 10), method='fast.heur.simple')`): 10 vertices "
        "all degree 3, Σd=30. Simple 3-regular graph (connectivity NOT "
        "enforced by FAST_HEUR_SIMPLE).",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "seed": 8_660_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 15,
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_fastheur_r_undirected_all_isolated",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=rep(0, 5), method='fast.heur.simple')`): five isolated "
        "vertices; ecount must be 0.",
        "algo": "degree_sequence_game_fast_heur_simple",
        "params": {
            "out_degrees": [0, 0, 0, 0, 0],
            "in_degrees": None,
            "seed": 8_660_003,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "ecount": 0,
            "out_degrees": [0, 0, 0, 0, 0],
            "in_degrees": None,
            "is_simple": True,
        },
    },
]

# ALGO-GN-027: degree_sequence_game (CONFIGURATION_SIMPLE method).
# rigraph exposes this as `sample_degseq(out.deg, in.deg=NULL,
# method="configuration.simple")` (also accepting a directed signature).
# CONFIGURATION_SIMPLE rejection-samples a uniform simple graph with
# the exact degree sequence via stub-matching with two-swap FY and
# restart-on-collision; expected restart count grows as exp(O((Σd/n)²))
# so fixtures stay at moderate density. RNG state is not portable.
DEGREE_SEQUENCE_CONFIG_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_configsimple_r_undirected_n10_descending",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=c(4,3,3,3,2,2,2,2,2,1), "
        "method='configuration.simple')`): moderately skewed descending "
        "sequence on 10 vertices, Σd=24.",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [4, 3, 3, 3, 2, 2, 2, 2, 2, 1],
            "in_degrees": None,
            "seed": 8_670_001,
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
        "case": "degseq_configsimple_r_3regular_n8",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=rep(3, 8), method='configuration.simple')`): 8 "
        "vertices all degree 3, Σd=24. Density Σd/n=3 keeps the "
        "rejection sampler tractable.",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "seed": 8_670_002,
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
        "case": "degseq_configsimple_r_undirected_all_isolated",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=rep(0, 5), method='configuration.simple')`): five "
        "isolated vertices; ecount must be 0 (early-exit branch).",
        "algo": "degree_sequence_game_configuration_simple",
        "params": {
            "out_degrees": [0, 0, 0, 0, 0],
            "in_degrees": None,
            "seed": 8_670_003,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "ecount": 0,
            "out_degrees": [0, 0, 0, 0, 0],
            "in_degrees": None,
            "is_simple": True,
        },
    },
]

# ALGO-GN-028: degree_sequence_game (EDGE_SWITCHING_SIMPLE method).
# rigraph exposes this as `sample_degseq(out.deg, in.deg=NULL,
# method="edge.switching.simple")` (and directed). Two-phase:
# deterministic Havel-Hakimi / Kleitman-Wang INDEX seed, then 10·|E|
# edge-switching MCMC trials. Linear-in-|E| cost makes it the
# preferred sampler for dense degree sequences. RNG not portable —
# pins vcount, ecount=Σd/2 or Σout, exact degree match, is_simple.
DEGREE_SEQUENCE_EDGE_SWITCHING_SIMPLE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_edge_switching_r_undirected_n10_skewed_dense",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=c(5,4,4,3,3,3,2,2,2,2), "
        "method='edge.switching.simple')`): n=10, Σd=30, density "
        "Σd/n=3 — dense regime where EDGE_SWITCHING_SIMPLE "
        "outperforms CONFIGURATION_SIMPLE.",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [5, 4, 4, 3, 3, 3, 2, 2, 2, 2],
            "in_degrees": None,
            "seed": 8_680_001,
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
        "case": "degseq_edge_switching_r_3regular_n10",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=rep(3, 10), method='edge.switching.simple')`): "
        "3-regular on 10 vertices, density Σd/n=3 — exercise the "
        "MCMC rewire kernel.",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "seed": 8_680_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 15,
            "out_degrees": [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
            "in_degrees": None,
            "is_simple": True,
        },
    },
    {
        "case": "degseq_edge_switching_r_directed_n6_skewed",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=c(3,2,2,1,1,1), in.deg=c(2,2,2,1,2,1), "
        "method='edge.switching.simple')`): directed skewed n=6, "
        "Σ=10.",
        "algo": "degree_sequence_game_edge_switching_simple",
        "params": {
            "out_degrees": [3, 2, 2, 1, 1, 1],
            "in_degrees": [2, 2, 2, 1, 2, 1],
            "seed": 8_680_003,
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

# ALGO-GN-025: degree_sequence_game (VL method). rigraph exposes this
# as `sample_degseq(out.deg, method = "vl")` (undirected only). VL
# samples a connected, simple undirected graph realising the given
# degree sequence — manifest pins vcount, ecount=Σd/2, exact degree
# match, simplicity, weak connectivity. RNG state is not portable to
# Rust's SplitMix64 — invariants only.
DEGREE_SEQUENCE_VL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "degseq_vl_r_undirected_n10_descending",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=c(4,3,3,3,3,2,2,2,1,1), method='vl')`): "
        "all-positive descending degree sequence, Σd=24.",
        "algo": "degree_sequence_game_vl",
        "params": {
            "degrees": [4, 3, 3, 3, 3, 2, 2, 2, 1, 1],
            "seed": 8_650_001,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 12,
            "degrees": [4, 3, 3, 3, 3, 2, 2, 2, 1, 1],
            "is_simple": True,
            "is_connected": True,
        },
    },
    {
        "case": "degseq_vl_r_3regular_n10",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=rep(3, 10), method='vl')`): 10 vertices all degree 3, "
        "Σd=30. Connected simple 3-regular graph.",
        "algo": "degree_sequence_game_vl",
        "params": {
            "degrees": [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
            "seed": 8_650_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "ecount": 15,
            "degrees": [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
            "is_simple": True,
            "is_connected": True,
        },
    },
    {
        "case": "degseq_vl_r_undirected_all_isolated",
        "origin": "constructed (mirrors `sample_degseq("
        "out.deg=rep(0, 5), method='vl')`): five isolated vertices; "
        "ecount must be 0.",
        "algo": "degree_sequence_game_vl",
        "params": {
            "degrees": [0, 0, 0, 0, 0],
            "seed": 8_650_003,
        },
        "expected": {
            "vcount": 5,
            "directed": False,
            "ecount": 0,
            "degrees": [0, 0, 0, 0, 0],
            "is_simple": True,
            "is_connected": True,
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

# ALGO-GN-010: sbm_game. Mirrors R `sample_sbm(n, pref.matrix, block.sizes,
# directed, loops)`. RNG state is not portable across implementations, so
# each fixture pins parameter values and bands the structural invariants:
#   * vcount = sum(block_sizes) (exact);
#   * directed matches the flag;
#   * ecount lies in a generous band around the model mean;
#   * is_simple (R's sample_sbm produces simple graphs);
#   * when the pref matrix is block-diagonal, every edge stays
#     on-diagonal (encoded via `diagonal_only_pref: true`).
SBM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "sbm_r_two_blocks_balanced_assortative",
        "origin": "constructed (mirrors R sample_sbm(n=20, "
        "pref.matrix=rbind(c(0.35, 0.05), c(0.05, 0.35)), "
        "block.sizes=c(10, 10), directed=FALSE, loops=FALSE)): canonical "
        "two-community SBM",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [[0.35, 0.05], [0.05, 0.35]],
            "block_sizes": [10, 10],
            "directed": False,
            "loops": False,
            "multiple": False,
            "seed": 10_200_001,
        },
        "expected": {
            "vcount": 20,
            "directed": False,
            "is_simple": True,
            "ecount_min": 18,
            "ecount_max": 75,
        },
    },
    {
        "case": "sbm_r_four_blocks_block_diagonal",
        "origin": "constructed (mirrors R sample_sbm with sizes=c(8, 8, 8, 8), "
        "block-diagonal pref (in-block 0.4, off-diagonal 0.0)): every "
        "realised edge stays inside a block — checks block_of(u) == "
        "block_of(v) invariant",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [
                [0.4, 0.0, 0.0, 0.0],
                [0.0, 0.4, 0.0, 0.0],
                [0.0, 0.0, 0.4, 0.0],
                [0.0, 0.0, 0.0, 0.4],
            ],
            "block_sizes": [8, 8, 8, 8],
            "directed": False,
            "loops": False,
            "multiple": False,
            "seed": 10_200_002,
        },
        "expected": {
            "vcount": 32,
            "directed": False,
            "is_simple": True,
            "ecount_min": 20,
            "ecount_max": 90,
            "diagonal_only_pref": True,
        },
    },
    {
        "case": "sbm_r_two_blocks_with_loops",
        "origin": "constructed (mirrors R sample_sbm with sizes=c(15, 15), "
        "uniform pref=0.2, undirected, loops=TRUE): on-diagonal block "
        "may produce self-loops; not necessarily simple",
        "algo": "sbm_game",
        "params": {
            "pref_matrix": [[0.2, 0.2], [0.2, 0.2]],
            "block_sizes": [15, 15],
            "directed": False,
            "loops": True,
            "multiple": False,
            "seed": 10_200_003,
        },
        "expected": {
            "vcount": 30,
            "directed": False,
            "is_simple": False,
            "ecount_min": 40,
            "ecount_max": 160,
        },
    },
]

# ALGO-GN-011: hsbm_game. Mirrors R `sample_hierarchical_sbm(n, m, rho, C, p)`
# (see references/rigraph/tests/testthat/test-games.R). RNG state is
# not portable, so fixtures pin corner-case probabilities (p=0 or p=1)
# for exact ecounts, plus one mid-density band fixture.
HSBM_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hsbm_r_three_clusters_p0_band",
        "origin": "constructed (mirrors R sample_hierarchical_sbm(n=20, m=10, "
        "rho=c(0.3, 0.3, 0.4), C=symmetric mix, p=0)): two macros each with "
        "three micro-clusters; ecount stays within a model-determined band",
        "algo": "hsbm_game",
        "params": {
            "n": 20,
            "m": 10,
            "rho": [0.3, 0.3, 0.4],
            "c": [[1.0, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5]],
            "p": 0.0,
            "seed": 11_020_001,
        },
        "expected": {
            "vcount": 20,
            "directed": False,
            "is_simple": True,
            "ecount_min": 10,
            "ecount_max": 80,
        },
    },
    {
        "case": "hsbm_r_one_cluster_per_block_p1",
        "origin": "constructed (mirrors R sample_hierarchical_sbm(10, 5, rho=1, "
        "C=matrix(0), p=1)): two macros each with a single micro-cluster, "
        "no intra edges (C=0), full inter K_{5,5}=25 — exactly 25 edges",
        "algo": "hsbm_game",
        "params": {
            "n": 10,
            "m": 5,
            "rho": [1.0],
            "c": [[0.0]],
            "p": 1.0,
            "seed": 11_020_002,
        },
        "expected": {
            "vcount": 10,
            "directed": False,
            "is_simple": True,
            "ecount_min": 25,
            "ecount_max": 25,
        },
    },
    {
        "case": "hsbm_r_two_macros_p_half_band",
        "origin": "constructed (mirrors R sample_hierarchical_sbm(n=20, m=10, "
        "rho=c(0.5, 0.5), C=block-diag (0.2, 0.1), p=0.5)): mid-density "
        "two-macro fixture; ecount stays within a wide model band",
        "algo": "hsbm_game",
        "params": {
            "n": 20,
            "m": 10,
            "rho": [0.5, 0.5],
            "c": [[0.2, 0.1], [0.1, 0.2]],
            "p": 0.5,
            "seed": 11_020_003,
        },
        "expected": {
            "vcount": 20,
            "directed": False,
            "is_simple": True,
            "ecount_min": 25,
            "ecount_max": 130,
        },
    },
]

# ALGO-GN-011: hsbm_list_game. Mirrors R sample_hierarchical_sbm called
# with per-macro `m`, `rho`, and `C` lists (the "HSBM with list arguments
# works" testthat block at references/rigraph/tests/testthat/test-games.R
# lines 650-767). Two of the three fixtures mirror the test that uses
# m=c(3, 10, 5, 3) with C all-zero or all-one — those give exact ecounts.
HSBM_LIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hsbm_list_r_uniform_equivalent_band",
        "origin": "constructed (mirrors R sample_hierarchical_sbm called with "
        "m=rep(10, 5) and rho/C replicated 5 times — the 'HSBM with list "
        "arguments works' equivalence test): five identical macros, "
        "p=0 stays within a model band",
        "algo": "hsbm_list_game",
        "params": {
            "n": 50,
            "m_list": [10, 10, 10, 10, 10],
            "rho_list": [
                [0.3, 0.3, 0.4],
                [0.3, 0.3, 0.4],
                [0.3, 0.3, 0.4],
                [0.3, 0.3, 0.4],
                [0.3, 0.3, 0.4],
            ],
            "c_list": [
                [[1.0, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5]],
                [[1.0, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5]],
                [[1.0, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5]],
                [[1.0, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5]],
                [[1.0, 0.5, 0.0], [0.5, 0.0, 0.5], [0.0, 0.5, 0.5]],
            ],
            "p": 0.0,
            "seed": 11_120_001,
        },
        "expected": {
            "vcount": 50,
            "directed": False,
            "is_simple": True,
            "ecount_min": 30,
            "ecount_max": 220,
        },
    },
    {
        "case": "hsbm_list_r_mixed_p1_intra_zero",
        "origin": "constructed (mirrors R 'g_hsbm5' with m=c(3, 10, 5, 3), "
        "rho/C as four separate lists with all-zero C entries, p=1): "
        "intra-macro edges are all zero, inter-macro is full bipartite "
        "between every macro pair — sum_{i<j} m_i*m_j = "
        "3*10+3*5+3*3+10*5+10*3+5*3 = 149",
        "algo": "hsbm_list_game",
        "params": {
            "n": 21,
            "m_list": [3, 10, 5, 3],
            "rho_list": [
                [1.0 / 3.0, 2.0 / 3.0],
                [0.3, 0.3, 0.4],
                [1.0],
                [2.0 / 3.0, 1.0 / 3.0],
            ],
            "c_list": [
                [[0.0, 0.0], [0.0, 0.0]],
                [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
                [[0.0]],
                [[0.0, 0.0], [0.0, 0.0]],
            ],
            "p": 1.0,
            "seed": 11_120_002,
        },
        "expected": {
            "vcount": 21,
            "directed": False,
            "is_simple": True,
            "ecount_min": 149,
            "ecount_max": 149,
        },
    },
    {
        "case": "hsbm_list_r_mixed_p0_intra_one",
        "origin": "constructed (mirrors R 'g_hsbm7' with the same m=c(3, 10, "
        "5, 3) shape but all-ones C and p=0): intra-macro is K_{m_i}, no "
        "inter — K_3+K_10+K_5+K_3 = 3+45+10+3 = 61 edges",
        "algo": "hsbm_list_game",
        "params": {
            "n": 21,
            "m_list": [3, 10, 5, 3],
            "rho_list": [
                [1.0 / 3.0, 2.0 / 3.0],
                [0.3, 0.3, 0.4],
                [1.0],
                [2.0 / 3.0, 1.0 / 3.0],
            ],
            "c_list": [
                [[1.0, 1.0], [1.0, 1.0]],
                [[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
                [[1.0]],
                [[1.0, 1.0], [1.0, 1.0]],
            ],
            "p": 0.0,
            "seed": 11_120_003,
        },
        "expected": {
            "vcount": 21,
            "directed": False,
            "is_simple": True,
            "ecount_min": 61,
            "ecount_max": 61,
        },
    },
]

# ALGO-GN-012: chung_lu_game. R exposes `sample_chung_lu(weights,
# in_weights = NULL, loops = FALSE, variant = c("original", "maxent",
# "nr"))` in references/rigraph/R/games.R:3100-3138. The test suite
# fixtures live at references/rigraph/tests/testthat/test-games.R:174-198
# ("sample_chung_lu works") — they apply the same weight vector
# c(3, 3, 2, 2, 1, 1) across all three variants with loops=FALSE,
# asserting is_simple. RNG state is not portable, so the ecount band
# is wide; structural invariants (vcount, directed, is_simple when
# loops=FALSE) are pinned.
CHUNG_LU_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "chung_lu_r_small_no_multi",
        "origin": "mirrors test-games.R:175 sample_chung_lu(c(3,3,2,2,1,1)) "
        "with default loops=FALSE + variant='original' (the R default "
        "for sample_chung_lu): asserts !any_multiple.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [3.0, 3.0, 2.0, 2.0, 1.0, 1.0],
            "in_weights": None,
            "loops": False,
            "variant": "original",
            "seed": 12_020_001,
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
        "case": "chung_lu_r_no_loop_original",
        "origin": "mirrors test-games.R:178-183 — variant='original', "
        "loops=FALSE: expect_true(is_simple(...)).",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [3.0, 3.0, 2.0, 2.0, 1.0, 1.0],
            "in_weights": None,
            "loops": False,
            "variant": "original",
            "seed": 12_020_002,
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
        "case": "chung_lu_r_no_loop_maxent",
        "origin": "mirrors test-games.R:185-190 — variant='maxent', "
        "loops=FALSE.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [3.0, 3.0, 2.0, 2.0, 1.0, 1.0],
            "in_weights": None,
            "loops": False,
            "variant": "maxent",
            "seed": 12_020_003,
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
        "case": "chung_lu_r_no_loop_nr",
        "origin": "mirrors test-games.R:192-197 — variant='nr', "
        "loops=FALSE.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [3.0, 3.0, 2.0, 2.0, 1.0, 1.0],
            "in_weights": None,
            "loops": False,
            "variant": "nr",
            "seed": 12_020_004,
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
        "case": "chung_lu_r_zero_weights_directed_empty",
        "origin": "constructed (mirrors sample_chung_lu(rep(0, 4), "
        "in_weights=rep(0, 4))): directed zero-weight graph has no "
        "edges regardless of variant.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [0.0, 0.0, 0.0, 0.0],
            "in_weights": [0.0, 0.0, 0.0, 0.0],
            "loops": True,
            "variant": "original",
            "seed": 12_020_005,
        },
        "expected": {
            "vcount": 4,
            "directed": True,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 0,
        },
    },
    {
        "case": "chung_lu_r_directed_doc_example",
        "origin": "mirrors games.R:3104 sample_chung_lu(c(1,3,2,1), "
        "c(2,1,2,2)): the R rdoc example. Directed, defaults to "
        "variant='original' loops=FALSE.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 3.0, 2.0, 1.0],
            "in_weights": [2.0, 1.0, 2.0, 2.0],
            "loops": False,
            "variant": "original",
            "seed": 12_020_006,
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
        "case": "chung_lu_r_directed_doc_example_maxent",
        "origin": "mirrors games.R:3109 sample_chung_lu(c(1,3,2,1), "
        "c(2,1,2,2), variant='maxent'): second R rdoc example.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 3.0, 2.0, 1.0],
            "in_weights": [2.0, 1.0, 2.0, 2.0],
            "loops": False,
            "variant": "maxent",
            "seed": 12_020_007,
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
        "case": "chung_lu_r_seven_weights_loops_original",
        "origin": "mirrors sample_chung_lu(c(1, 0, 2.5, 2, 3, 2, 1.5), "
        "loops=TRUE, variant='original'): same weights as the C "
        "test_unit fixture but exercised under R's defaults.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [1.0, 0.0, 2.5, 2.0, 3.0, 2.0, 1.5],
            "in_weights": None,
            "loops": True,
            "variant": "original",
            "seed": 12_020_008,
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
        "case": "chung_lu_r_eight_weights_band",
        "origin": "constructed (mirrors sample_chung_lu(c(2,2,2,2,2,2,2,2), "
        "loops=FALSE, variant='original')): uniform weights of 2 across "
        "n=8 → q_ij = 4/16 = 0.25 for every pair; expected edges "
        "≈ 0.25*C(8,2) = 7; band is wide.",
        "algo": "chung_lu_game",
        "params": {
            "out_weights": [2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0],
            "in_weights": None,
            "loops": False,
            "variant": "original",
            "seed": 12_020_009,
        },
        "expected": {
            "vcount": 8,
            "directed": False,
            "is_simple": True,
            "ecount_min": 0,
            "ecount_max": 28,
        },
    },
]


# ALGO-GN-013 (static_fitness_game). rigraph exposes
# sample_fitness(no.of.edges, fitness.out, fitness.in=NULL,
# loops=FALSE, multiple=FALSE) — see references/rigraph/R/games.R.
# Cases mirror the binding's documented happy paths. RNG state is not
# portable; structural invariants only.
STATIC_FITNESS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "static_fitness_r_zero_edges",
        "origin": "constructed (mirrors sample_fitness(0, c(1,2,3,4,5))): "
        "five isolated vertices.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 0,
            "fitness_out": [1.0, 2.0, 3.0, 4.0, 5.0],
            "fitness_in": None,
            "loops": False,
            "multiple": False,
            "seed": 12_021_001,
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
        "case": "static_fitness_r_undirected_simple",
        "origin": "constructed (mirrors sample_fitness(12, c(1,2,3,4,5,6))): "
        "default loops=FALSE, multiple=FALSE → simple undirected graph. "
        "Capacity C(6,2) = 15 ≥ 12.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 12,
            "fitness_out": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "fitness_in": None,
            "loops": False,
            "multiple": False,
            "seed": 12_021_002,
        },
        "expected": {
            "vcount": 6,
            "directed": False,
            "is_simple": True,
            "ecount_min": 12,
            "ecount_max": 12,
        },
    },
    {
        "case": "static_fitness_r_undirected_loops_multi",
        "origin": "constructed (mirrors sample_fitness(15, c(1,1,1,1,1,1), "
        "loops=TRUE, multiple=TRUE)): permissive sampling.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 15,
            "fitness_out": [1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            "fitness_in": None,
            "loops": True,
            "multiple": True,
            "seed": 12_021_003,
        },
        "expected": {
            "vcount": 6,
            "directed": False,
            "ecount_min": 15,
            "ecount_max": 15,
        },
    },
    {
        "case": "static_fitness_r_directed_simple",
        "origin": "constructed (mirrors sample_fitness(15, "
        "c(1,2,3,4,5), c(5,4,3,2,1))): directed simple. Capacity "
        "5*4 = 20 ≥ 15.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 15,
            "fitness_out": [1.0, 2.0, 3.0, 4.0, 5.0],
            "fitness_in": [5.0, 4.0, 3.0, 2.0, 1.0],
            "loops": False,
            "multiple": False,
            "seed": 12_021_004,
        },
        "expected": {
            "vcount": 5,
            "directed": True,
            "is_simple": True,
            "ecount_min": 15,
            "ecount_max": 15,
        },
    },
    {
        "case": "static_fitness_r_directed_loops_multi",
        "origin": "constructed (mirrors sample_fitness(40, c(1,2,3,4), "
        "c(4,3,2,1), loops=TRUE, multiple=TRUE)): directed, fully "
        "permissive.",
        "algo": "static_fitness_game",
        "params": {
            "no_of_edges": 40,
            "fitness_out": [1.0, 2.0, 3.0, 4.0],
            "fitness_in": [4.0, 3.0, 2.0, 1.0],
            "loops": True,
            "multiple": True,
            "seed": 12_021_005,
        },
        "expected": {
            "vcount": 4,
            "directed": True,
            "ecount_min": 40,
            "ecount_max": 40,
        },
    },
]


# ALGO-GN-013 (static_power_law_game). rigraph exposes
# sample_fitness_pl(no.of.nodes, no.of.edges, exponent.out,
# exponent.in=-1, loops=FALSE, multiple=FALSE,
# finite.size.correction=TRUE). Negative exponent.in selects
# undirected (passes -1 down to igraph C). Cases mirror the C-test
# happy paths under R defaults.
STATIC_POWER_LAW_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "static_power_law_r_zero_edges",
        "origin": "constructed (mirrors sample_fitness_pl(10, 0, 2.5)): "
        "isolated graph regardless of exponent.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 10,
            "no_of_edges": 0,
            "exponent_out": 2.5,
            "exponent_in": None,
            "loops": False,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_021_101,
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
        "case": "static_power_law_r_undirected_simple",
        "origin": "constructed (mirrors sample_fitness_pl(40, 60, 2.5)): "
        "undirected simple under R defaults.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 40,
            "no_of_edges": 60,
            "exponent_out": 2.5,
            "exponent_in": None,
            "loops": False,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_021_102,
        },
        "expected": {
            "vcount": 40,
            "directed": False,
            "is_simple": True,
            "ecount_min": 60,
            "ecount_max": 60,
        },
    },
    {
        "case": "static_power_law_r_undirected_loops_only",
        "origin": "constructed (mirrors sample_fitness_pl(60, 80, 2.2, "
        "loops=TRUE)): undirected, loops allowed, no parallel edges.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 60,
            "no_of_edges": 80,
            "exponent_out": 2.2,
            "exponent_in": None,
            "loops": True,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_021_103,
        },
        "expected": {
            "vcount": 60,
            "directed": False,
            "is_simple": False,
            "no_multi_edges": True,
            "ecount_min": 80,
            "ecount_max": 80,
        },
    },
    {
        "case": "static_power_law_r_directed_simple",
        "origin": "constructed (mirrors sample_fitness_pl(50, 80, 2.5, "
        "exponent.in=2.8)): directed simple — non-negative exponent.in "
        "selects directed shape.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 50,
            "no_of_edges": 80,
            "exponent_out": 2.5,
            "exponent_in": 2.8,
            "loops": False,
            "multiple": False,
            "finite_size_correction": True,
            "seed": 12_021_104,
        },
        "expected": {
            "vcount": 50,
            "directed": True,
            "is_simple": True,
            "ecount_min": 80,
            "ecount_max": 80,
        },
    },
    {
        "case": "static_power_law_r_directed_multi",
        "origin": "constructed (mirrors sample_fitness_pl(40, 100, 2.5, "
        "exponent.in=2.5, loops=TRUE, multiple=TRUE)): directed fully "
        "permissive.",
        "algo": "static_power_law_game",
        "params": {
            "no_of_nodes": 40,
            "no_of_edges": 100,
            "exponent_out": 2.5,
            "exponent_in": 2.5,
            "loops": True,
            "multiple": True,
            "finite_size_correction": True,
            "seed": 12_021_105,
        },
        "expected": {
            "vcount": 40,
            "directed": True,
            "ecount_min": 100,
            "ecount_max": 100,
        },
    },
]


# ALGO-CN-001: ring (R-igraph factory `make_ring(n, directed, mutual,
# circular)`). Fully deterministic; expected edges in raw upstream
# order, harness compares undirected fixtures via canonicalised
# multisets.
RING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "ring_r_path_p4_undirected",
        "origin": "rigraph make_ring(n=4, directed=FALSE, mutual=FALSE, "
        "circular=FALSE)",
        "algo": "ring_graph",
        "params": {"n": 4, "directed": False, "mutual": False, "circular": False},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3]],
        },
    },
    {
        "case": "ring_r_cycle_c4_undirected",
        "origin": "rigraph make_ring(n=4, directed=FALSE, mutual=FALSE, "
        "circular=TRUE)",
        "algo": "ring_graph",
        "params": {"n": 4, "directed": False, "mutual": False, "circular": True},
        "expected": {
            "vcount": 4,
            "ecount": 4,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 0]],
        },
    },
    {
        "case": "ring_r_directed_cycle_c5",
        "origin": "rigraph make_ring(n=5, directed=TRUE, mutual=FALSE, "
        "circular=TRUE)",
        "algo": "ring_graph",
        "params": {"n": 5, "directed": True, "mutual": False, "circular": True},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": True,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [4, 0]],
        },
    },
    {
        "case": "ring_r_two_vertex_undirected_cycle_parallel",
        "origin": "rigraph make_ring(n=2, directed=FALSE, mutual=FALSE, "
        "circular=TRUE) — degenerate case: forward + wrap collapse to two "
        "parallel (0,1) edges",
        "algo": "ring_graph",
        "params": {"n": 2, "directed": False, "mutual": False, "circular": True},
        "expected": {
            "vcount": 2,
            "ecount": 2,
            "directed": False,
            "edges": [[0, 1], [1, 0]],
        },
    },
]


STAR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "star_r_undirected_k1_3",
        "origin": "rigraph make_star(n=4, mode='undirected') — "
        "K1,3 with vertex 0 as the centre",
        "algo": "star_graph",
        "params": {"n": 4, "mode": "Undirected", "center": 0},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": False,
            "edges": [[1, 0], [2, 0], [3, 0]],
        },
    },
    {
        "case": "star_r_out_center_zero",
        "origin": "rigraph make_star(n=5, mode='out') — "
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
        "case": "star_r_in_center_two",
        "origin": "rigraph make_star(n=5, mode='in', center=3) — "
        "directed in-star with non-zero centre, R 1-based index 3 → C/Rust 2",
        "algo": "star_graph",
        "params": {"n": 5, "mode": "In", "center": 2},
        "expected": {
            "vcount": 5,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 2], [1, 2], [3, 2], [4, 2]],
        },
    },
    {
        "case": "star_r_mutual_center_zero_n3",
        "origin": "rigraph make_star(n=3, mode='mutual') — "
        "minimal mutual star; forward arc (centre→leaf) emitted first per leaf",
        "algo": "star_graph",
        "params": {"n": 3, "mode": "Mutual", "center": 0},
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": True,
            "edges": [[0, 1], [1, 0], [0, 2], [2, 0]],
        },
    },
]


WHEEL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "wheel_r_undirected_w4",
        "origin": "rigraph make_wheel(n=4, mode='undirected') — "
        "minimal non-degenerate wheel: 3 spokes + 3 rim edges",
        "algo": "wheel_graph",
        "params": {"n": 4, "mode": "Undirected", "center": 0},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [
                [1, 0], [2, 0], [3, 0],
                [1, 2], [2, 3], [3, 1],
            ],
        },
    },
    {
        "case": "wheel_r_out_w5_center_two",
        "origin": "rigraph make_wheel(n=5, mode='out', center=3) — "
        "directed out-wheel with non-zero centre (R 1-based 3 → C/Rust 2); "
        "rim skips the centre",
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
        "case": "wheel_r_mutual_w4_center_zero",
        "origin": "rigraph make_wheel(n=4, mode='mutual') — "
        "spokes mutual then rim forward + reverse-discovery",
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
        "case": "wheel_r_two_vertex_self_loop",
        "origin": "rigraph make_wheel(n=2, mode='undirected') — "
        "degenerate: rim collapses to 1-cycle (self-loop on vertex 1)",
        "algo": "wheel_graph",
        "params": {"n": 2, "mode": "Undirected", "center": 0},
        "expected": {
            "vcount": 2,
            "ecount": 2,
            "directed": False,
            "edges": [[1, 0], [1, 1]],
        },
    },
]

KARY_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "kary_tree_r_binary_seven_undirected",
        "origin": "rigraph make_tree(n=7, children=2, mode='undirected') — "
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
        "case": "kary_tree_r_binary_seven_in",
        "origin": "rigraph make_tree(n=7, children=2, mode='in') — "
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
        "case": "kary_tree_r_chain_path_six",
        "origin": "rigraph make_tree(n=6, children=1, mode='out') — "
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
        "case": "kary_tree_r_singleton",
        "origin": "rigraph make_tree(n=1, ...) — singleton root, no edges",
        "algo": "kary_tree",
        "params": {"n": 1, "children": 3, "mode": "Out"},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]

SYMMETRIC_TREE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "symmetric_tree_r_binary_two_two_undirected",
        "origin": "rigraph make_symmetric_tree(branches=c(2,2), mode='undirected') — "
        "perfect binary depth 2, mirrors make_tree(7,2)",
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
        "case": "symmetric_tree_r_three_two_in",
        "origin": "rigraph make_symmetric_tree(branches=c(3,2), mode='in') — "
        "root with 3 kids, each with 2 grandkids, child→parent arcs",
        "algo": "symmetric_tree",
        "params": {"branches": [3, 2], "mode": "In"},
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
        "case": "symmetric_tree_r_chain_three_ones_undirected",
        "origin": "rigraph make_symmetric_tree(branches=c(1,1,1), mode='undirected') — "
        "linear chain of 4 vertices",
        "algo": "symmetric_tree",
        "params": {"branches": [1, 1, 1], "mode": "Undirected"},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3],
            ],
        },
    },
    {
        "case": "symmetric_tree_r_empty_branches_singleton",
        "origin": "rigraph make_symmetric_tree(branches=integer(0)) — "
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
        "case": "regular_tree_r_h1_k3_out",
        "origin": "rigraph make_tree(...) Bethe variant make_regular_tree(h=1, k=3, mode='out') — "
        "root with 3 leaves",
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
        "case": "regular_tree_r_h2_k3_in",
        "origin": "rigraph make_regular_tree(h=2, k=3, mode='in') — "
        "Bethe lattice, child→parent arcs",
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
        "case": "regular_tree_r_h2_k3_undirected",
        "origin": "rigraph make_regular_tree(h=2, k=3, mode='undirected') — "
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
        "case": "regular_tree_r_h3_k2_undirected",
        "origin": "rigraph make_regular_tree(h=3, k=2, mode='undirected') — "
        "degenerate k=2 case (branches=[2,1,1]); 1+2+2+2=7 vertices",
        "algo": "regular_tree",
        "params": {"h": 3, "k": 2, "mode": "Undirected"},
        "expected": {
            "vcount": 7,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [1, 3], [2, 4], [3, 5], [4, 6],
            ],
        },
    },
]


HYPERCUBE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "hypercube_r_n1_undirected",
        "origin": "rigraph make_hypercube(1, directed=FALSE) — Q_1 = K_2",
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
        "case": "hypercube_r_n2_undirected",
        "origin": "rigraph make_hypercube(2, directed=FALSE) — Q_2 is the 4-cycle",
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
        "case": "hypercube_r_n3_undirected",
        "origin": "rigraph make_hypercube(3, directed=FALSE) — Q_3 with 8 vertices and 12 edges",
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
        "case": "hypercube_r_n4_undirected",
        "origin": "rigraph make_hypercube(4, directed=FALSE) — Q_4 with 16 vertices and 32 edges",
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


GENERALIZED_PETERSEN_MANIFEST: List[Dict[str, Any]] = [
    # Note: R-igraph does not expose `igraph::make_generalized_petersen`
    # directly. We use make_graph('Petersen') as the only available
    # canonical G(n,k) labeling — make_graph('Dodecahedron') is also
    # isomorphic to G(10,2) but uses an embedded-polytope vertex layout
    # whose edge multiset differs from the canonical one, so we cannot
    # include it without isomorphism-based comparison.
    {
        "case": "generalized_petersen_r_g_5_2_petersen",
        "origin": "rigraph make_graph('Petersen') — classic Petersen G(5,2) via famous DB",
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
    # Note: R-igraph does not expose `igraph::make_circulant` directly
    # at this version. We cover the same two canonical specializations
    # used in the python manifest via rigraph's make_ring (single-shift
    # cycle) and make_full_graph (every distinct undirected shift =
    # K_n). Both functions dispatch to the same igraph_ring / igraph_full
    # C entry points used internally by circulant for these inputs, so
    # the canonical edge multiset must agree.
    {
        "case": "circulant_r_c_6_shifts_1_ring",
        "origin": "rigraph make_ring(6, directed=FALSE, circular=TRUE) — equivalent to circulant(6, [1], False) = C_6",
        "algo": "circulant",
        "params": {"n": 6, "shifts": [1], "directed": False},
        "expected": {
            "vcount": 6,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [0, 5],
            ],
        },
    },
    {
        "case": "circulant_r_k5_shifts_1_2_full",
        "origin": "rigraph make_full_graph(5, directed=FALSE) — equivalent to circulant(5, [1, 2], False) = K_5",
        "algo": "circulant",
        "params": {"n": 5, "shifts": [1, 2], "directed": False},
        "expected": {
            "vcount": 5,
            "ecount": 10,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3], [0, 4],
                [1, 2], [1, 3], [1, 4],
                [2, 3], [2, 4],
                [3, 4],
            ],
        },
    },
]


DE_BRUIJN_MANIFEST: List[Dict[str, Any]] = [
    # rigraph exposes `igraph::make_de_bruijn_graph(m, n)`, which
    # dispatches directly to the upstream `igraph_de_bruijn()` C entry
    # point. Edge emission order is therefore identical to the C and
    # python-igraph oracles: for each vertex i ∈ [0, m^n), arcs
    # (i, (i*m mod vcount) + b) for b ∈ [0, m).
    {
        "case": "de_bruijn_r_b_2_2",
        "origin": "rigraph make_de_bruijn_graph(m=2, n=2) — 4 vertices, 8 directed arcs",
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
        "case": "de_bruijn_r_b_3_2",
        "origin": "rigraph make_de_bruijn_graph(m=3, n=2) — 9 vertices, 27 directed arcs",
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
    # rigraph exposes `igraph::make_kautz_graph(m, n)`, which dispatches
    # directly to the upstream `igraph_kautz()` C entry point. Edge order
    # therefore matches the C and python-igraph oracles byte-for-byte:
    # source-major, target-ascending over the m valid successor letters.
    {
        "case": "kautz_r_m2_n1",
        "origin": "rigraph make_kautz_graph(m=2, n=1) — 6 vertices, 12 directed arcs",
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
        "case": "kautz_r_m2_n2",
        "origin": "rigraph make_kautz_graph(m=2, n=2) — 12 vertices, 24 directed arcs",
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
]


SQUARE_LATTICE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "square_lattice_r_dim_three_path",
        "origin": "rigraph make_lattice(c(3), nei=1, circular=FALSE) — path P_3",
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
        "case": "square_lattice_r_dim_3x3_grid",
        "origin": "rigraph make_lattice(c(3,3), nei=1, circular=FALSE) — 3x3 grid",
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
        "case": "square_lattice_r_dim_2x2_four_cycle",
        "origin": "rigraph make_lattice(c(2,2), nei=1) — 2x2 is 4-cycle",
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
        "case": "square_lattice_r_dim_2x2x2_cube",
        "origin": "rigraph make_lattice(c(2,2,2), nei=1) — Q_3 cube",
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
        "case": "hamming_r_n1_q3_is_k3",
        "origin": "rigraph make_hamming_graph(1, 3, directed=FALSE) — H(1,3) = K_3",
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
        "case": "hamming_r_n2_q3_undirected",
        "origin": "rigraph make_hamming_graph(2, 3, directed=FALSE) — H(2,3), 9v/18e",
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
        "case": "hamming_r_n3_q2_equals_hypercube",
        "origin": "rigraph make_hamming_graph(3, 2, directed=FALSE) — H(3,2) ≡ Q_3",
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
        "case": "hamming_r_n2_q4_undirected",
        "origin": "rigraph make_hamming_graph(2, 4, directed=FALSE) — H(2,4), 16v/48e",
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


FULL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_r_n4_ud_noloops",
        "origin": "mirrors R-igraph make_full_graph(4, directed=FALSE, loops=FALSE) — undirected K_4, 6 edges (dispatches to igraph_full)",
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
        "case": "full_r_n3_d_loops",
        "origin": "mirrors R-igraph make_full_graph(3, directed=TRUE, loops=TRUE) — directed K_3 + self-loops, 9 arcs",
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
    {
        "case": "full_r_n5_ud_loops",
        "origin": "mirrors R-igraph make_full_graph(5, directed=FALSE, loops=TRUE) — undirected K_5 + self-loops, 15 edges",
        "algo": "full_graph",
        "params": {"n": 5, "directed": False, "loops": True},
        "expected": {
            "vcount": 5,
            "ecount": 15,
            "directed": False,
            "edges": [
                [0, 0], [0, 1], [0, 2], [0, 3], [0, 4],
                [1, 1], [1, 2], [1, 3], [1, 4],
                [2, 2], [2, 3], [2, 4],
                [3, 3], [3, 4],
                [4, 4],
            ],
        },
    },
]


# ALGO-CN-025 — `make_full_citation_graph` (R-igraph) dispatches directly
# to `igraph_full_citation`. Fixtures mirror the four cases the upstream C
# unit test asserts (n=4 ud + d, n=1 d, n=0 d) so the R lane stays in
# lock-step with `igraph_full_citation.c`. Emission order matches the
# upstream C function: `(i, j)` for every `j < i`.
FULL_CITATION_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_citation_r_n4_undirected",
        "origin": "mirrors R-igraph make_full_graph_citation_graph(n=4, directed=FALSE) — K_4 (matches igraph_full_citation.c case 1)",
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
        "case": "full_citation_r_n4_directed",
        "origin": "mirrors R-igraph make_full_graph_citation_graph(n=4, directed=TRUE) — complete DAG with arcs i->j for every j<i",
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
        "case": "full_citation_r_n1_directed",
        "origin": "mirrors R-igraph make_full_graph_citation_graph(n=1, directed=TRUE) — edgeless singleton",
        "algo": "full_citation",
        "params": {"n": 1, "directed": True},
        "expected": {
            "vcount": 1,
            "ecount": 0,
            "directed": True,
            "edges": [],
        },
    },
]


# ALGO-CN-026 — `make_full_multipartite` (R-igraph, references/rigraph/R/make.R:2740)
# dispatches via `full_multipartite_impl` to `igraph_full_multipartite`.
# `make_full_bipartite_graph(n1, n2)` is the bipartite shorthand. Both
# wrappers expose the same mode argument as the C entry point
# ("all" / "out" / "in"). Fixtures cover (a) the canonical undirected
# K_{2,3} bipartite from the R help example, (b) the same partitions
# under directed mode "out", and (c) the directed K_{2,2,2} from the R
# help page, with all-mutual arcs.
FULL_MULTIPARTITE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "full_multipartite_r_bipartite_2_3_undirected",
        "origin": "mirrors R-igraph make_full_bipartite_graph(2, 3) — undirected K_{2,3} with 6 edges (rigraph help example for bipartite)",
        "algo": "full_multipartite",
        "params": {"partitions": [2, 3], "directed": False, "mode": "all"},
        "expected": {
            "vcount": 5,
            "ecount": 6,
            "directed": False,
            "edges": [
                [0, 2], [0, 3], [0, 4],
                [1, 2], [1, 3], [1, 4],
            ],
            "types": [0, 0, 1, 1, 1],
        },
    },
    {
        "case": "full_multipartite_r_bipartite_2_3_directed_out",
        "origin": "mirrors R-igraph make_full_bipartite_graph(2, 3, directed=TRUE, mode='out') — 6 arcs partition 0 → partition 1",
        "algo": "full_multipartite",
        "params": {"partitions": [2, 3], "directed": True, "mode": "out"},
        "expected": {
            "vcount": 5,
            "ecount": 6,
            "directed": True,
            "edges": [
                [0, 2], [0, 3], [0, 4],
                [1, 2], [1, 3], [1, 4],
            ],
            "types": [0, 0, 1, 1, 1],
        },
    },
    {
        "case": "full_multipartite_r_tripartite_2_2_2_directed_out",
        "origin": "mirrors R-igraph make_full_multipartite(c(2,2,2), directed=TRUE, mode='out') — K_{2,2,2} directed forward, 12 arcs (rigraph help example for multipartite)",
        "algo": "full_multipartite",
        "params": {"partitions": [2, 2, 2], "directed": True, "mode": "out"},
        "expected": {
            "vcount": 6,
            "ecount": 12,
            "directed": True,
            "edges": [
                [0, 2], [0, 3], [0, 4], [0, 5],
                [1, 2], [1, 3], [1, 4], [1, 5],
                [2, 4], [2, 5],
                [3, 4], [3, 5],
            ],
            "types": [0, 0, 1, 1, 2, 2],
        },
    },
]


# ALGO-CN-027 — `make_turan(n, r)` (R-igraph, references/rigraph/R/make.R:2790)
# dispatches via `turan_impl` to `igraph_turan`. Like upstream the R
# wrapper returns the graph plus a `types` attribute. Fixtures pick the
# canonical R-help shapes: (a) T(10, 3) — the unique 4-balanced
# tripartition, (b) T(7, 4) — quotient 1, remainder 3, sizes [2,2,2,1],
# (c) T(9, 3) — balanced [3,3,3] giving K_{3,3,3} a.k.a. the "complete
# tripartite 9".
TURAN_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "turan_r_n10_r3",
        "origin": "mirrors R-igraph make_turan(n=10, r=3) — partitions [4,3,3], 33 edges across three balanced parts (rigraph help example)",
        "algo": "turan",
        "params": {"n": 10, "r": 3},
        "expected": {
            "vcount": 10,
            "ecount": 33,
            "directed": False,
            "edges": [
                [0, 4], [0, 5], [0, 6], [0, 7], [0, 8], [0, 9],
                [1, 4], [1, 5], [1, 6], [1, 7], [1, 8], [1, 9],
                [2, 4], [2, 5], [2, 6], [2, 7], [2, 8], [2, 9],
                [3, 4], [3, 5], [3, 6], [3, 7], [3, 8], [3, 9],
                [4, 7], [4, 8], [4, 9],
                [5, 7], [5, 8], [5, 9],
                [6, 7], [6, 8], [6, 9],
            ],
            "types": [0, 0, 0, 0, 1, 1, 1, 2, 2, 2],
        },
    },
    {
        "case": "turan_r_n7_r4",
        "origin": "mirrors R-igraph make_turan(n=7, r=4) — partitions [2,2,2,1] (quotient 1, remainder 3), 18 edges",
        "algo": "turan",
        "params": {"n": 7, "r": 4},
        "expected": {
            "vcount": 7,
            "ecount": 18,
            "directed": False,
            "edges": [
                [0, 2], [0, 3], [0, 4], [0, 5], [0, 6],
                [1, 2], [1, 3], [1, 4], [1, 5], [1, 6],
                [2, 4], [2, 5], [2, 6],
                [3, 4], [3, 5], [3, 6],
                [4, 6], [5, 6],
            ],
            "types": [0, 0, 1, 1, 2, 2, 3],
        },
    },
    {
        "case": "turan_r_n9_r3_balanced",
        "origin": "mirrors R-igraph make_turan(n=9, r=3) — balanced partitions [3,3,3] giving the complete tripartite K_{3,3,3}, 27 edges",
        "algo": "turan",
        "params": {"n": 9, "r": 3},
        "expected": {
            "vcount": 9,
            "ecount": 27,
            "directed": False,
            "edges": [
                [0, 3], [0, 4], [0, 5], [0, 6], [0, 7], [0, 8],
                [1, 3], [1, 4], [1, 5], [1, 6], [1, 7], [1, 8],
                [2, 3], [2, 4], [2, 5], [2, 6], [2, 7], [2, 8],
                [3, 6], [3, 7], [3, 8],
                [4, 6], [4, 7], [4, 8],
                [5, 6], [5, 7], [5, 8],
            ],
            "types": [0, 0, 0, 1, 1, 1, 2, 2, 2],
        },
    },
]


# ALGO-CN-028 — `make_chordal_ring(n, w, directed=FALSE)` (R-igraph,
# references/rigraph/R/make.R:2334) dispatches to
# `igraph_extended_chordal_ring`. python-igraph has no binding, so the
# fixture set is two-source: C `.out` rows from
# `tests/unit/igraph_extended_chordal_ring.c` plus R helper-shaped
# fixtures mirroring `make_chordal_ring` calls. All three R fixtures use
# the undirected default and stay within textbook ranges (period 1 / 3
# / 2, nodes ≤ 10). Edge order follows the algorithm's emission order
# (cycle first, then per-vertex per-row chords) and the conformance
# dispatcher canonicalises (min, max) before multiset comparison.
EXTENDED_CHORDAL_RING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "extended_chordal_ring_r_n8_period1_offset2",
        "origin": "mirrors R-igraph make_chordal_ring(n=8, w=matrix(2), directed=FALSE) — 8-cycle plus 8 chord edges at offset +2 (period 1)",
        "algo": "extended_chordal_ring",
        "params": {
            "nodes": 8,
            "w": [[2]],
            "directed": False,
        },
        "expected": {
            "vcount": 8,
            "ecount": 16,
            "directed": False,
            "edges": [
                # 8-cycle backbone (emission order)
                [0, 1], [1, 2], [2, 3], [3, 4],
                [4, 5], [5, 6], [6, 7], [7, 0],
                # chord (i, (i+2) mod 8)
                [0, 2], [1, 3], [2, 4], [3, 5],
                [4, 6], [5, 7], [6, 0], [7, 1],
            ],
        },
    },
    {
        "case": "extended_chordal_ring_r_n9_period3",
        "origin": "mirrors R-igraph make_chordal_ring(n=9, w=matrix(c(2,3,4), 1, 3), directed=FALSE) — 9-cycle plus 9 chord edges with period 3 offsets {2,3,4}",
        "algo": "extended_chordal_ring",
        "params": {
            "nodes": 9,
            "w": [[2, 3, 4]],
            "directed": False,
        },
        "expected": {
            "vcount": 9,
            "ecount": 18,
            "directed": False,
            "edges": [
                # 9-cycle backbone
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5],
                [5, 6], [6, 7], [7, 8], [8, 0],
                # chord per vertex (period 3): offsets [2,3,4]
                [0, 2], [1, 4], [2, 6], [3, 5], [4, 7],
                [5, 0], [6, 8], [7, 1], [8, 3],
            ],
        },
    },
    {
        "case": "extended_chordal_ring_r_n10_period2_two_rows",
        "origin": "mirrors R-igraph make_chordal_ring(n=10, w=matrix(c(2,3,4,5), 2, 2), directed=FALSE) — 10-cycle plus 20 chord edges from a 2×2 offset matrix (period 2)",
        "algo": "extended_chordal_ring",
        "params": {
            "nodes": 10,
            "w": [[2, 3], [4, 5]],
            "directed": False,
        },
        "expected": {
            "vcount": 10,
            "ecount": 30,
            "directed": False,
            "edges": [
                # 10-cycle backbone
                [0, 1], [1, 2], [2, 3], [3, 4], [4, 5],
                [5, 6], [6, 7], [7, 8], [8, 9], [9, 0],
                # per-vertex chords: row0 then row1, mpos = i mod 2
                [0, 2], [0, 4],   # i=0: +2, +4
                [1, 4], [1, 6],   # i=1: +3, +5
                [2, 4], [2, 6],   # i=2: +2, +4
                [3, 6], [3, 8],   # i=3: +3, +5
                [4, 6], [4, 8],   # i=4: +2, +4
                [5, 8], [5, 0],   # i=5: +3, +5
                [6, 8], [6, 0],   # i=6: +2, +4
                [7, 0], [7, 2],   # i=7: +3, +5
                [8, 0], [8, 2],   # i=8: +2, +4
                [9, 2], [9, 4],   # i=9: +3, +5
            ],
        },
    },
]


# ALGO-CN-015 — `make_line_graph` (R-igraph) dispatches to
# `igraph_linegraph`. Fixtures cover the textbook small shapes —
# triangle, K_4, star — that an R-bindings user would build with
# `make_full_graph` / `make_star`.
LINEGRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "linegraph_r_k3_undirected",
        "origin": "rigraph make_line_graph(make_full_graph(3)) — L(K_3) = K_3 on three L-vertices",
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
    {
        "case": "linegraph_r_k4_undirected",
        "origin": "rigraph make_line_graph(make_full_graph(4)) — L(K_4) on 6 L-vertices, 12 L-edges (3-regular)",
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
        "case": "linegraph_r_star_s5",
        "origin": "rigraph make_line_graph(make_star(5, mode='undirected')) — L of an n-leaf star is K_{n} (here 4 leaves → K_4)",
        "algo": "linegraph",
        "graph_factory": lambda: ig.Graph(
            5,
            edges=[(0, 1), (0, 2), (0, 3), (0, 4)],
            directed=False,
        ),
        "params": {},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [[0, 1], [0, 2], [1, 2], [0, 3], [1, 3], [2, 3]],
        },
    },
]


# ALGO-CN-016 — `make_from_prufer(prufer)` (rigraph) dispatches to the
# same C `igraph_from_prufer`. rigraph's `test-trees.R` round-trips
# `to_prufer(make_tree(13, 3))` through `make_from_prufer`; we mirror
# that here directly, plus two upstream-C fixtures so the R extractor
# yields three independent shapes.
# ALGO-CN-017 — `tree_from_parent_vector_impl(parents, type)` (rigraph
# auto-bound) shifts the input by -1 inside R then dispatches to C
# `igraph_tree_from_parent_vector`. rigraph's `test-aaa-auto.R` snapshot
# uses `parents = c(-1, 1, 2, 3)` which after the shift becomes the
# C-level vector `[-2, 0, 1, 2]` — a chain 0→1→2→3. We mirror both the
# OUT and IN snapshots here (in 0-based form), plus an undirected
# variant that also matches rigraph's natural conversion.
TREE_FROM_PARENT_VECTOR_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "tree_from_parent_vector_r_chain_out",
        "origin": "rigraph test-aaa-auto.R snapshot: tree_from_parent_vector_impl(parents=c(-1,1,2,3)) — internal -1 shift gives C parents=[-2,0,1,2]; OUT mode prints 1->2 2->3 3->4 (0-based: 0→1 1→2 2→3)",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-2, 0, 1, 2], "mode": "out"},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": True,
            "edges": [[0, 1], [1, 2], [2, 3]],
        },
    },
    {
        "case": "tree_from_parent_vector_r_chain_in",
        "origin": "rigraph test-aaa-auto.R snapshot: tree_from_parent_vector_impl(parents=c(-1,1,2,3), type='in') — IN mode prints 2->1 3->2 4->3 (0-based: 1→0 2→1 3→2)",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-2, 0, 1, 2], "mode": "in"},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": True,
            "edges": [[1, 0], [2, 1], [3, 2]],
        },
    },
    {
        "case": "tree_from_parent_vector_r_chain_undirected",
        "origin": "rigraph extension of the snapshot: same chain decoded undirected → path P_4 with canonical edges {(0,1),(1,2),(2,3)}",
        "algo": "tree_from_parent_vector",
        "params": {"parents": [-2, 0, 1, 2], "mode": "undirected"},
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3]],
        },
    },
]


# ALGO-CN-018 — `graph_from_lcf(n, shifts, repeats)` (rigraph, formerly
# `graph.lcf`) dispatches to the same C `igraph_lcf`. rigraph's
# `test-aaa-auto.R` snapshot exercises the Franklin graph and a single
# trivial fixture; we add Heawood and a pure-cycle case to round out the
# topology coverage.
LCF_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "lcf_r_franklin",
        "origin": "rigraph graph_from_lcf(12, c(5, -5), 6) — Franklin graph, the canonical LCF showcase (12 vertices, 18 edges)",
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
        "case": "lcf_r_heawood",
        "origin": "rigraph graph_from_lcf(14, c(5, -5), 7) — Heawood graph (14 vertices, 21 edges, bipartite cubic, girth 6)",
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
        "case": "lcf_r_repeats_zero_pure_cycle",
        "origin": "rigraph graph_from_lcf(5, c(1, 2, 3), 0) — repeats=0 skips the entire chord pass; result is C_5",
        "algo": "lcf",
        "params": {"n": 5, "shifts": [1, 2, 3], "repeats": 0},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3], [3, 4], [0, 4]],
        },
    },
]


# ALGO-CN-019 — rigraph's `test-aaa-auto.R` auto-test snapshot exercises
# both `mycielski_graph(k=3)` (yielding C_5) and `mycielskian(P_3)` /
# `mycielskian(P_3, k=2)` (yielding 7v/9e and 15v/34e). Edge lists below
# were copied from the snapshot file (`_snaps/aaa-auto.md`) and converted
# from R's 1-based to 0-based indexing.
MYCIELSKI_GRAPH_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "mycielski_graph_r_k3_c5",
        "origin": "rigraph test-aaa-auto.R snapshot: mycielski_graph_impl(k=3) → C_5 (IGRAPH U--- 5 5, edges 1-2 1-4 2-3 3-5 4-5)",
        "algo": "mycielski_graph",
        "params": {"k": 3},
        "expected": {
            "vcount": 5,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [0, 3], [1, 2], [2, 4], [3, 4]],
        },
    },
]


# rigraph's `make_graph("<name>")` user-facing API resolves the string
# branch to `famous_impl()` in `R/aaa-auto.R`, which calls the same
# `igraph_famous` C entry point. `test-aaa-auto.R` exercises this path
# (e.g. `make_graph("Zachary")` at line 10875). We pin a handful of
# canonical witnesses here; edge lists for the larger graphs are dropped
# in favour of structural counts so the JSON stays manageable.
FAMOUS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "famous_r_bull",
        "origin": "rigraph make_graph('Bull') — dispatches to famous_impl → igraph_famous",
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
        "case": "famous_r_petersen",
        "origin": "rigraph make_graph('Petersen') — 10v/15e Petersen graph",
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
        "case": "famous_r_zachary_counts",
        "origin": "rigraph make_graph('Zachary') — exercised directly by test-aaa-auto.R:10875",
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
        "case": "famous_r_tetrahedral_alias",
        "origin": "rigraph make_graph('Tetrahedral') — alias of 'Tetrahedron' (K_4)",
        "algo": "famous",
        "params": {"name": "Tetrahedral"},
        "expected": {
            "vcount": 4,
            "ecount": 6,
            "directed": False,
            "edges": [[0, 3], [1, 3], [2, 3], [0, 1], [1, 2], [0, 2]],
        },
    },
]


# rigraph exposes `make_graph(edges, n=NA, directed=TRUE)` and
# `graph(edges, n=NA, directed=TRUE)` as wrappers over `igraph_create`.
# rigraph converts to 1-based vertex IDs internally; the fixtures here
# carry the 0-based form to match the JSON wire shape used by the other
# two extractors.
CREATE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "create_r_make_graph_n_zero_infers",
        "origin": "rigraph make_graph(c(1,2, 2,3, 3,4, 3,3), directed=FALSE) — 0-based: [(0,1),(1,2),(2,3),(2,2)] n=0",
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
        "case": "create_r_explicit_n_kept",
        "origin": "rigraph make_graph(c(1,2), n=5, directed=FALSE) — n=5 kept, 4 isolated vertices",
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
        "case": "create_r_directed_two_arcs",
        "origin": "rigraph make_graph(c(1,2, 2,1), directed=TRUE) — 0-based: [(0,1),(1,0)] both arcs",
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
        "case": "create_r_empty_with_isolated",
        "origin": "rigraph make_graph(numeric(0), n=4, directed=FALSE) — 4 isolated vertices, no edges",
        "algo": "create",
        "params": {
            "edges": [],
            "n": 4,
            "directed": False,
        },
        "expected": {
            "vcount": 4,
            "ecount": 0,
            "directed": False,
            "edges": [],
        },
    },
    {
        "case": "create_r_parallel_edges_kept",
        "origin": "rigraph make_graph(c(1,2, 1,2, 1,2), directed=FALSE) — 0-based 3 parallel edges between 0 and 1",
        "algo": "create",
        "params": {
            "edges": [[0, 1], [0, 1], [0, 1]],
            "n": 0,
            "directed": False,
        },
        "expected": {
            "vcount": 2,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [0, 1], [0, 1]],
        },
    },
    {
        "case": "create_r_path4_via_create",
        "origin": "rigraph make_graph(c(1,2, 2,3, 3,4), directed=FALSE) — 0-based path P_4",
        "algo": "create",
        "params": {
            "edges": [[0, 1], [1, 2], [2, 3]],
            "n": 0,
            "directed": False,
        },
        "expected": {
            "vcount": 4,
            "ecount": 3,
            "directed": False,
            "edges": [[0, 1], [1, 2], [2, 3]],
        },
    },
]


# rigraph's `triangular_lattice_impl` (R/aaa-auto.R:724) is an auto-bound
# 1:1 wrapper for the C `igraph_triangular_lattice` entry point. The
# rigraph snapshots in tests/testthat/_snaps/aaa-auto.md cover two
# canonical cases on the 2x2 rectangle shape (1-based R labels translated
# to 0-based here). These mirror the python-igraph testTriangularLattice
# witnesses and ground the R lane against the same upstream invariants.
TRIANGULAR_LATTICE_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "triangular_lattice_r_2x2_undirected",
        "origin": "rigraph triangular_lattice_impl(dimvector=c(2,2)) — _snaps/aaa-auto.md:220",
        "algo": "triangular_lattice",
        "params": {"dims": [2, 2], "directed": False, "mutual": False},
        "expected": {
            "vcount": 4,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 1], [0, 3], [0, 2], [1, 3], [2, 3]],
        },
    },
    {
        "case": "triangular_lattice_r_2x2_directed_mutual",
        "origin": "rigraph triangular_lattice_impl(dimvector=c(2,2), directed=TRUE, mutual=TRUE) — _snaps/aaa-auto.md:229",
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
        "case": "triangular_lattice_r_triangle_side_3_undirected",
        "origin": "synthetic — triangular_lattice_impl(dimvector=c(3)) triangle side 3 (cross-checked against C upstream)",
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
]


# rigraph exposes `graph_from_atlas(n)` (also aliased `atlas(n)`) in
# `R/make_graph.R` — the binding ultimately calls the same C
# `igraph_atlas` entry point through `graph_from_atlas_impl`. The R man
# page (`man/graph_from_atlas.Rd`) and `test-aaa-auto.R` cover indexing,
# so we mirror small canonical witnesses here.
ATLAS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "atlas_r_null0",
        "origin": "rigraph graph_from_atlas(0) — null graph on 0 vertices (R uses 0-based indexing here, like C)",
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
        "case": "atlas_r_triangle",
        "origin": "rigraph graph_from_atlas(7) — the triangle K_3, last 3-vertex entry",
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
        "case": "atlas_r_k4",
        "origin": "rigraph graph_from_atlas(18) — complete K_4, last 4-vertex entry",
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
        "case": "atlas_r_k7_last",
        "origin": "rigraph graph_from_atlas(1252) — complete K_7, the final atlas entry",
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
        "case": "mycielskian_r_p3_one_iteration",
        "origin": "rigraph test-aaa-auto.R snapshot: mycielskian_impl(graph=P_3) → 7v/9e (default k=1)",
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
        "case": "mycielskian_r_p3_two_iterations",
        "origin": "rigraph test-aaa-auto.R snapshot: mycielskian_impl(graph=P_3, k=2) → 15v/34e",
        "graph_factory": lambda: ig.Graph(n=3, edges=[(0, 1), (1, 2)], directed=False),
        "algo": "mycielskian",
        "params": {"k": 2},
        "expected": {
            "vcount": 15,
            "ecount": 34,
            "directed": False,
            "edges": [
                [0, 1], [1, 2], [0, 4], [1, 3], [1, 5], [2, 4], [3, 6], [4, 6], [5, 6],
                [0, 8], [1, 7], [1, 9], [2, 8], [0, 11], [4, 7], [1, 10], [3, 8], [1, 12], [5, 8],
                [2, 11], [4, 9], [3, 13], [6, 10], [4, 13], [6, 11], [5, 13], [6, 12],
                [7, 14], [8, 14], [9, 14], [10, 14], [11, 14], [12, 14], [13, 14],
            ],
        },
    },
]


PRUFER_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "from_prufer_r_make_tree_13_3_roundtrip",
        "origin": "rigraph test-trees.R: make_tree(13, 3, undirected) → to_prufer → make_from_prufer round-trip; prufer = [1,1,1,0,2,2,2,0,3,3,3]",
        "algo": "from_prufer",
        "params": {"prufer": [1, 1, 1, 0, 2, 2, 2, 0, 3, 3, 3]},
        "expected": {
            "vcount": 13,
            "ecount": 12,
            "directed": False,
            "edges": [
                [0, 1], [0, 2], [0, 3],
                [1, 4], [1, 5], [1, 6],
                [2, 7], [2, 8], [2, 9],
                [3, 10], [3, 11], [3, 12],
            ],
        },
    },
    {
        "case": "from_prufer_r_seq_2323",
        "origin": "rigraph make_from_prufer([2,3,2,3]) — matches upstream igraph_from_prufer.c fixture 1",
        "algo": "from_prufer",
        "params": {"prufer": [2, 3, 2, 3]},
        "expected": {
            "vcount": 6,
            "ecount": 5,
            "directed": False,
            "edges": [[0, 2], [1, 3], [2, 3], [2, 4], [3, 5]],
        },
    },
    {
        "case": "from_prufer_r_empty",
        "origin": "rigraph make_from_prufer(integer(0)) → P_2 (matches upstream igraph_from_prufer.c fixture 3)",
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


# ALGO-CN-029 — rigraph `graph_from_adjacency_matrix(A, mode=..., diag=...,
# weighted=NULL)` dispatches to `igraph_adjacency()`. Modes mirror the C
# enum (directed/undirected/max/min/plus/upper/lower); `diag` is a
# boolean (TRUE = once, FALSE = NO_LOOPS); there is no native TWICE
# wrapper, so TWICE-collapse semantics are exercised only via the C and
# python sides. The fixtures here are computed by running the upstream
# wrapper on the same M3 / M3_SYM matrices the C unit test uses.
ADJACENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "adjacency_r_3x3_directed_no_diag",
        "origin": "rigraph graph_from_adjacency_matrix([[4,2,0],[3,0,4],[0,5,6]], mode='directed', diag=FALSE) — 14 off-diagonal arcs (matches the C M3+DIRECTED+NO_LOOPS fixture)",
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
        "case": "adjacency_r_3x3_undirected_diag",
        "origin": "rigraph graph_from_adjacency_matrix([[4,2,0],[2,0,4],[0,4,6]], mode='undirected', diag=TRUE) — symmetric matrix with loops once; matches C M3_SYM+UNDIRECTED+LOOPS_ONCE",
        "algo": "adjacency",
        "params": {
            "matrix": [[4, 2, 0], [2, 0, 4], [0, 4, 6]],
            "mode": "undirected",
            "loops": "once",
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
        "case": "adjacency_r_3x3_lower_no_diag",
        "origin": "rigraph graph_from_adjacency_matrix(M3, mode='lower', diag=FALSE) — only the strict lower triangle contributes: M[1,0]=3 plus M[2,1]=5 (matches C M3+LOWER+NO_LOOPS)",
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


# ALGO-CN-030 — rigraph `graph_from_adjacency_matrix(A, mode=...,
# diag=..., weighted=TRUE)` dispatches to `igraph_weighted_adjacency()`
# when the `weighted` argument is non-NULL. As with the integer wrapper,
# `diag=TRUE` corresponds to LOOPS_ONCE and there is no native TWICE mode
# (TWICE-halving is exercised via the C and python sides instead). The
# fixtures here mirror the M3 family used by the C unit test, captured
# from the R wrapper.
WEIGHTED_ADJACENCY_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "weighted_adjacency_r_3x3_directed_no_diag",
        "origin": "rigraph graph_from_adjacency_matrix([[2.0,0.5,0],[1.5,0,2.0],[0,2.5,3.0]], mode='directed', diag=FALSE, weighted=TRUE) — 4 weighted off-diagonal arcs",
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
        "case": "weighted_adjacency_r_3x3_undirected_diag",
        "origin": "rigraph graph_from_adjacency_matrix(M3_SYM, mode='undirected', diag=TRUE, weighted=TRUE) — diagonal kept un-halved (R wrapper only exposes ONCE)",
        "algo": "weighted_adjacency",
        "params": {
            "matrix": [[2.0, 0.5, 0.0], [0.5, 0.0, 2.0], [0.0, 2.0, 3.0]],
            "mode": "undirected",
            "loops": "once",
        },
        "expected": {
            "vcount": 3,
            "ecount": 4,
            "directed": False,
            # row-major lower: i=0 diag 2.0; i=1 (1,0)=0.5; i=2 diag 3.0, (2,1)=2.0
            "edges": [[0, 0], [0, 1], [2, 2], [1, 2]],
            "weights": [2.0, 0.5, 3.0, 2.0],
        },
    },
    {
        "case": "weighted_adjacency_r_3x3_lower_no_diag",
        "origin": "rigraph graph_from_adjacency_matrix(M3, mode='lower', diag=FALSE, weighted=TRUE) — strict lower triangle: M[1,0]=1.5, M[2,1]=2.5; M[2,0]=0 skipped",
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
            "edges": [[0, 1], [1, 2]],
            "weights": [1.5, 2.5],
        },
    },
]


# ALGO-CL-001: coloring. R-igraph exposes `greedy_vertex_coloring`.
COLORING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "coloring_r_is_valid_star",
        "origin": "hand-computed: valid 2-coloring of star S_4 (center=0 color 0, leaves color 1)",
        "graph_factory": lambda: ig.Graph(n=5, edges=[(0, 1), (0, 2), (0, 3), (0, 4)], directed=False),
        "algo": "coloring",
        "params": {"check": "is_vertex_coloring", "colors": [0, 1, 1, 1, 1]},
        "expected": True,
    },
    {
        "case": "coloring_r_greedy_k4_cn",
        "origin": "hand-computed: K4 (χ=4), CN heuristic must use exactly 4 colors",
        "graph_factory": lambda: ig.Graph(n=4, edges=[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], directed=False),
        "algo": "coloring",
        "params": {"check": "greedy_valid", "heuristic": "colored_neighbors"},
        "expected": {"valid": True, "max_colors": 4},
    },
]


# ALGO-CL-002: chordal. R-igraph exposes `is_chordal`.
CHORDAL_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "chordal_r_tree_chordal",
        "origin": "hand-computed: any tree is chordal",
        "graph_factory": lambda: ig.Graph(n=5, edges=[(0, 1), (1, 2), (1, 3), (3, 4)], directed=False),
        "algo": "chordal",
        "params": {"check": "is_chordal"},
        "expected": {"chordal": True, "fill_in": []},
    },
    {
        "case": "chordal_r_cycle6_not_chordal",
        "origin": "hand-computed: C_6 is NOT chordal",
        "graph_factory": lambda: ig.Graph(n=6, edges=[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)], directed=False),
        "algo": "chordal",
        "params": {"check": "is_chordal"},
        "expected": {"chordal": False},
    },
]


# ALGO-CL-003: matching. R-igraph exposes matching functions.
MATCHING_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "matching_r_valid_k22",
        "origin": "hand-computed: perfect matching on K_{2,2} (0↔2, 1↔3)",
        "graph_factory": lambda: ig.Graph(n=4, edges=[(0, 2), (0, 3), (1, 2), (1, 3)], directed=False),
        "algo": "matching",
        "params": {"check": "is_matching", "matching": [2, 3, 0, 1]},
        "expected": True,
    },
    {
        "case": "matching_r_invalid_self",
        "origin": "hand-computed: vertex 0 matched to itself is invalid",
        "graph_factory": lambda: ig.Graph(n=3, edges=[(0, 1), (1, 2)], directed=False),
        "algo": "matching",
        "params": {"check": "is_matching", "matching": [0, -1, -1]},
        "expected": False,
    },
]


# ALGO-LO-001: layout. R-igraph exposes `layout_in_circle()`, `layout_as_star()`.
import math

LAYOUT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "layout_r_circle_5",
        "origin": "hand-computed: layout_circle on 5-vertex graph — regular pentagon on unit circle",
        "graph_factory": lambda: ig.Graph(n=5, edges=[(0, 1), (1, 2), (2, 3), (3, 4)], directed=False),
        "algo": "layout",
        "params": {"algorithm": "circle"},
        "expected": [
            [math.cos(2 * math.pi * i / 5), math.sin(2 * math.pi * i / 5)]
            for i in range(5)
        ],
    },
    {
        "case": "layout_r_star_center1",
        "origin": "hand-computed: layout_star on 3 vertices with center=1",
        "graph_factory": lambda: ig.Graph(n=3, edges=[(0, 1), (1, 2)], directed=False),
        "algo": "layout",
        "params": {"algorithm": "star", "center": 1},
        "expected": [
            [1.0, 0.0],
            [0.0, 0.0],
            [math.cos(math.pi), math.sin(math.pi)],
        ],
    },
]


# ALGO-SP-031: all_simple_paths. R-igraph exposes `all_simple_paths`.
ALL_SIMPLE_PATHS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "all_simple_paths_r_diamond",
        "origin": "hand-computed: diamond graph (K4 minus one edge), from=0 to=[3] — 3 paths",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "algo": "all_simple_paths",
        "params": {"from": 0, "to": [3], "mode": "all", "min_len": -1, "max_len": -1, "max_results": -1},
        "expected": [[0, 1, 2, 3], [0, 1, 3], [0, 2, 1, 3], [0, 2, 3]],
    },
    {
        "case": "all_simple_paths_r_maxlen2",
        "origin": "hand-computed: diamond graph, from=0 to=[3], maxlen=2 — only length-2 paths",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)],
            directed=False,
        ),
        "algo": "all_simple_paths",
        "params": {"from": 0, "to": [3], "mode": "all", "min_len": -1, "max_len": 2, "max_results": -1},
        "expected": [[0, 1, 3], [0, 2, 3]],
    },
]


# ALGO-SP-030: path_length_hist. R-igraph exposes `path.length.hist`.
PATH_LENGTH_HIST_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "path_length_hist_r_star5",
        "origin": "hand-computed: star graph with 5 vertices — center at distance 1 from all, leaves at distance 2 from each other",
        "graph_factory": lambda: ig.Graph(
            n=5,
            edges=[(0, 1), (0, 2), (0, 3), (0, 4)],
            directed=False,
        ),
        "algo": "path_length_hist",
        "params": {"directed": False},
        "expected": {"hist": [4.0, 6.0], "unconnected": 0.0},
    },
    {
        "case": "path_length_hist_r_disconnected",
        "origin": "hand-computed: two components {0-1} and {2-3} — hist=[2], unconnected=4",
        "graph_factory": lambda: ig.Graph(
            n=4,
            edges=[(0, 1), (2, 3)],
            directed=False,
        ),
        "algo": "path_length_hist",
        "params": {"directed": False},
        "expected": {"hist": [2.0], "unconnected": 4.0},
    },
]


# ALGO-PR-036: trussness. R-igraph exposes `trussness_impl`.
# The auto-generated R test only uses a path graph (P_3) which yields
# all-2 trussness. We add a second case with a triangle for coverage.
TRUSSNESS_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "trussness_r_path3",
        "origin": "R-igraph test-aaa-auto.R #162: path_graph_impl(n=3) — no triangles, all edges trussness 2",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (1, 2)],
            directed=False,
        ),
        "algo": "trussness",
        "params": {},
        "expected": [2, 2],
    },
    {
        "case": "trussness_r_triangle",
        "origin": "R-igraph trussness_impl on triangle (K3) — all edges in one triangle, trussness 3",
        "graph_factory": lambda: ig.Graph(
            n=3,
            edges=[(0, 1), (0, 2), (1, 2)],
            directed=False,
        ),
        "algo": "trussness",
        "params": {},
        "expected": [3, 3, 3],
    },
]


# `power_law_fit` ≙ rigraph `fit_power_law`, which wraps the same igraph C core
# (0.10.16) as python-igraph. R is not installed in this environment, so the
# expected values are the authoritative igraph C golden output
# (`igraph_power_law_fit.out`, printf "%.5f") that rigraph reproduces verbatim
# via the shared core. Matched to that published 5-decimal precision. The two
# fixed-xmin cases here complement the auto-xmin cases harvested from from_c.
POWER_LAW_FIT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "r_continuous_fixed_xmin2",
        "origin": "rigraph fit_power_law (igraph C 0.10.16); igraph_power_law_fit.out block 2 "
        "(continuous data, xmin=2, force_continuous=0)",
        "dataset": "continuous",
        "xmin": 2,
        "force_continuous": False,
        "expected": {
            "continuous": True,
            "alpha": 2.81157,
            "xmin": 2.00000,
            "L": -463.92064,
            "D": 0.05091,
        },
    },
    {
        "case": "r_discrete_fixed_xmin2",
        "origin": "rigraph fit_power_law (igraph C 0.10.16); igraph_power_law_fit.out block 4 "
        "(discrete data, xmin=2, force_continuous=0)",
        "dataset": "discrete",
        "xmin": 2,
        "force_continuous": False,
        "expected": {
            "continuous": False,
            "alpha": 3.27157,
            "xmin": 2.00000,
            "L": -185.83215,
            "D": 0.04504,
        },
    },
]


# `dim_select` ≙ rigraph's `dim_select` (Zhu & Ghodsi 2006 profile-likelihood
# elbow). The R wrapper calls the same igraph C core as python-igraph (which
# does NOT expose this function). The anchor is rigraph's own deterministic
# test (`test-embedding.R`): `dim_select(1:100) == 50` — the symmetric ramp's
# elbow sits at the midpoint. Input is a plain numeric vector (not a graph),
# so the emit branch builds params from `sv` with a placeholder graph payload
# (mirrors the `power_law_fit` data-vector pattern).
DIM_SELECT_MANIFEST: List[Dict[str, Any]] = [
    {
        "case": "r_ramp_100",
        "origin": "rigraph test-embedding.R: dim_select(1:100) == 50 "
        "(equal-variance two-Gaussian profile-likelihood elbow at the midpoint)",
        "sv": [float(i) for i in range(1, 101)],
        "expected": 50,
    },
    {
        "case": "r_ramp_10",
        "origin": "rigraph dim_select(1:10) == 5 (symmetric ramp, midpoint elbow); "
        "same igraph C core as test-embedding.R",
        "sv": [float(i) for i in range(1, 11)],
        "expected": 5,
    },
]


ALGO_MANIFESTS: Dict[str, List[Dict[str, Any]]] = {
    "bfs": BFS_MANIFEST,
    "power_law_fit": POWER_LAW_FIT_MANIFEST,
    "dim_select": DIM_SELECT_MANIFEST,
    "count_isomorphisms_vf2": VF2_COUNT_MANIFEST,
    "count_subisomorphisms_vf2": SUBISO_COUNT_MANIFEST,
    "count_automorphisms": COUNT_AUTOMORPHISMS_MANIFEST,
    "automorphism_group": AUTOMORPHISM_GROUP_MANIFEST,
    "isomorphic_bliss": ISOMORPHIC_BLISS_MANIFEST,
    "isomorphic": ISOMORPHIC_GENERIC_MANIFEST,
    "subisomorphic": SUBISOMORPHIC_MANIFEST,
    "subisomorphic_lad": SUBISOMORPHIC_LAD_MANIFEST,
    "get_subisomorphisms_lad": GET_SUBISOMORPHISMS_LAD_MANIFEST,
    "community_to_membership": COMMUNITY_TO_MEMBERSHIP_MANIFEST,
    "reindex_membership": REINDEX_MEMBERSHIP_MANIFEST,
    "compare_communities": COMPARE_COMMUNITIES_MANIFEST,
    "split_join_distance": SPLIT_JOIN_DISTANCE_MANIFEST,
    "voronoi": VORONOI_MANIFEST,
    "ecc": ECC_PR031_MANIFEST,
    "rich_club_sequence": RICH_CLUB_MANIFEST,
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
    "is_perfect": IS_PERFECT_MANIFEST,
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
    "assortativity_values": ASSORT_VAL_MANIFEST,
    "get_shortest_path_astar": ASTAR_R_MANIFEST,
    "get_all_shortest_paths_dijkstra": ALL_SP_DIJKSTRA_R_MANIFEST,
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
    "all_st_cuts": ALL_ST_CUTS_MANIFEST,
    "all_st_mincuts": ALL_ST_MINCUTS_MANIFEST,
    "minimum_size_separators": MINIMUM_SIZE_SEPARATORS_MANIFEST,
    "cohesive_blocks": COHESIVE_BLOCKS_MANIFEST,
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
    "turan": TURAN_MANIFEST,
    "extended_chordal_ring": EXTENDED_CHORDAL_RING_MANIFEST,
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
    "adjacency": ADJACENCY_MANIFEST,
    "weighted_adjacency": WEIGHTED_ADJACENCY_MANIFEST,
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
        # `community_to_membership` is a pure dendrogram helper —
        # bypass graph_factory.
        if algo == "power_law_fit":
            data = list(_PLFIT_DATA[entry["dataset"]])
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": {"n": 1, "edges": [], "directed": False, "weights": None},
                "algo": algo,
                "params": {
                    "data": data,
                    "xmin": entry["xmin"],
                    "force_continuous": bool(entry["force_continuous"]),
                },
                "expected": entry["expected"],
            }
        elif algo == "dim_select":
            sv = [float(x) for x in entry["sv"]]
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": {"n": 1, "edges": [], "directed": False, "weights": None},
                "algo": algo,
                "params": {"sv": sv},
                "expected": entry["expected"],
            }
        elif algo == "community_to_membership":
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
            "turan",
            "extended_chordal_ring",
            "from_prufer",
            "tree_from_parent_vector",
            "lcf",
            "mycielski_graph",
            "famous",
            "atlas",
            "create",
            "triangular_lattice",
            "adjacency",
            "weighted_adjacency",
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
        elif algo == "assortativity_values":
            g = entry["graph_factory"]()
            graph_payload = graph_to_payload(g)
            values = [float(x) for x in entry["values"]]
            values_in = (
                [float(x) for x in entry["values_in"]]
                if entry.get("values_in") is not None
                else None
            )
            directed = bool(entry.get("directed", False))
            normalized = bool(entry.get("normalized", True))
            r = g.assortativity(
                values, values_in, directed=directed, normalized=normalized
            )
            # rigraph shares the C core; NaN encodes the undefined case.
            expected = None if (r is None or r != r) else float(r)
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": graph_payload,
                "algo": algo,
                "params": {
                    "values": values,
                    "values_in": values_in,
                    "weights": None,
                    "directed": directed,
                    "normalized": normalized,
                },
                "expected": expected,
            }
        elif algo == "get_all_shortest_paths_dijkstra":
            g = entry["graph_factory"]()
            graph_payload = graph_to_payload(g)
            source = int(entry["source"])
            weights_raw = entry.get("weights")
            if weights_raw is None:
                weights = [1.0] * g.ecount()
            else:
                weights = [float(x) for x in weights_raw]
            nrgeo = []
            for target in range(g.vcount()):
                paths = g.get_all_shortest_paths(
                    source, to=target, weights=weights, mode="all"
                )
                nrgeo.append(len(paths))
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": graph_payload,
                "algo": algo,
                "params": {
                    "source": source,
                    "weights": weights,
                    "mode": "all",
                },
                "expected": nrgeo,
            }
        elif algo == "get_shortest_path_astar":
            g = entry["graph_factory"]()
            graph_payload = graph_to_payload(g)
            fr = int(entry["from"])
            to = int(entry["to"])
            weights = entry.get("weights")
            mode_str = entry.get("mode", "all")
            vpath = g.get_shortest_path_astar(
                fr, to=to, weights=weights,
                heuristics=lambda _g, _v, _t: 0,
                mode=mode_str,
            )
            if len(vpath) == 0:
                expected = None
            elif len(vpath) == 1:
                expected = 0.0
            else:
                epath = g.get_shortest_path_astar(
                    fr, to=to, weights=weights,
                    heuristics=lambda _g, _v, _t: 0,
                    mode=mode_str,
                    output="epath",
                )
                if weights is not None:
                    expected = float(sum(weights[e] for e in epath))
                else:
                    expected = float(len(epath))
            payload = {
                "source": "r",
                "origin": entry["origin"],
                "graph": graph_payload,
                "algo": algo,
                "params": {
                    "from": fr,
                    "to": to,
                    "weights": [float(x) for x in weights] if weights else None,
                    "mode": mode_str,
                },
                "expected": expected,
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
