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

ALGO_MANIFESTS: Dict[str, List[Dict[str, Any]]] = {
    "bfs": BFS_MANIFEST,
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
