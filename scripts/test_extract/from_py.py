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
            "source": "py",
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
