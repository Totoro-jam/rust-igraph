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
        payload = {
            "source": "c",
            "origin": entry["origin"],
            "graph": graph_to_payload(g),
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
