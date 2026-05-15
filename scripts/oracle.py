#!/usr/bin/env python3
"""Live oracle bridging Rust tests to python-igraph.

Protocol (stdin -> stdout, both single-line JSON):

  request:
    {
      "graph": {"n": int, "edges": [[u, v], ...], "directed": bool,
                "weights": [float, ...] | null},
      "algo":  str,
      "params": {<algo-specific kwargs>}
    }

  response on success:
    {"ok": true,  "result": <algo-specific>, "version": "<lib version>"}

  response on failure:
    {"ok": false, "error":  "<message>",     "version": "<lib version>"}

Each algorithm landed by an AWU adds one branch to ``run()``. The protocol is
intentionally minimal so adding new branches is mechanical.

Usage:
    .venv/bin/python scripts/oracle.py < request.json
"""

from __future__ import annotations

import json
import sys
from typing import Any, Dict

import igraph as ig


def make_graph(payload: Dict[str, Any]) -> ig.Graph:
    """Build a python-igraph Graph from the canonical wire format."""
    n = int(payload["n"])
    edges = [tuple(e) for e in payload.get("edges", [])]
    directed = bool(payload.get("directed", False))
    g = ig.Graph(n=n, edges=edges, directed=directed)
    weights = payload.get("weights")
    if weights is not None:
        g.es["weight"] = list(weights)
    return g


def run(algo: str, g: ig.Graph, params: Dict[str, Any]) -> Any:
    """Dispatch the named algorithm. Add a branch per AWU."""
    if algo == "bfs":
        # Counterpart of igraph_bfs(). We return only the visit order, which is
        # what crates/igraph-algorithms/src/traversal/bfs.rs currently produces.
        # Full callback-driven BFS comes later (ALGO-TR-001).
        root = int(params["root"])
        # python-igraph's Graph.bfs returns (vids, layers, parents).
        order, _layers, _parents = g.bfs(root)
        return list(order)

    if algo == "dfs":
        # Counterpart of igraph_dfs(). ALGO-TR-002 returns only the
        # pre-order visit list (single root, unreachable=False).
        # python-igraph's Graph.dfs returns (vids, parents) for the
        # `mode='all'` default we use on undirected graphs. Convert
        # the AttributeList to a plain list of ints.
        root = int(params["root"])
        result = g.dfs(root)
        # Older python-igraph returns 2-tuple (vids, parents); newer
        # may return more — guard by indexing.
        order = result[0] if isinstance(result, tuple) else result
        return [int(v) for v in order]

    raise NotImplementedError(f"oracle has no branch for algo={algo!r}")


def main() -> int:
    raw = sys.stdin.read()
    try:
        req = json.loads(raw)
        g = make_graph(req["graph"])
        result = run(req["algo"], g, req.get("params", {}))
        sys.stdout.write(
            json.dumps({"ok": True, "result": result, "version": ig.__version__})
        )
        return 0
    except Exception as exc:  # noqa: BLE001 - oracle reports any error verbatim
        sys.stdout.write(
            json.dumps(
                {
                    "ok": False,
                    "error": f"{type(exc).__name__}: {exc}",
                    "version": ig.__version__,
                }
            )
        )
        return 1


if __name__ == "__main__":
    sys.exit(main())
