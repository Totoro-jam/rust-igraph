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

    if algo == "connected_components":
        # Counterpart of igraph_connected_components(_, _, _, _, IGRAPH_WEAK).
        # python-igraph's Graph.connected_components(mode='weak') returns
        # a VertexClustering whose .membership is the per-vertex component id.
        # Component ids are dense (0..count) but assigned in vertex-id order
        # by both implementations.
        cc = g.connected_components(mode="weak")
        return {"membership": list(cc.membership), "count": len(cc)}

    if algo == "eccentricity":
        # Counterpart of igraph_eccentricity(_, NULL_weights, _, igraph_vss_all(), IGRAPH_OUT).
        # python-igraph returns floats; cast to int.
        return [int(x) for x in g.eccentricity()]

    if algo == "radius":
        # Counterpart of igraph_radius(_, NULL_weights, _, IGRAPH_OUT).
        # Empty graph returns NaN upstream; we map to None.
        if g.vcount() == 0:
            return None
        return int(g.radius())

    if algo == "diameter":
        # Counterpart of igraph_diameter(_, NULL, _, NULL, NULL, NULL, NULL, _, true).
        if g.vcount() == 0:
            return None
        return int(g.diameter())

    if algo == "count_triangles":
        # python-igraph 0.11 doesn't expose `count_triangles` directly;
        # it offers `list_triangles()` which returns one tuple per
        # triangle. Length of that list is the count.
        return len(g.list_triangles())

    if algo == "transitivity_local_undirected":
        # Counterpart of igraph_transitivity_local_undirected(_, _, igraph_vss_all(),
        # IGRAPH_TRANSITIVITY_NAN). python-igraph returns a list of floats (NaN for
        # degree<2). Map NaN to None for `Vec<Option<f64>>` parity.
        vals = g.transitivity_local_undirected(mode="nan")
        return [None if (v != v) else float(v) for v in vals]

    if algo == "transitivity_undirected":
        # Counterpart of igraph_transitivity_undirected(_, &result, IGRAPH_TRANSITIVITY_NAN).
        # python-igraph returns NaN if there are no connected triples;
        # we encode that as None to match `Option<f64>`.
        v = g.transitivity_undirected(mode="nan")
        # `v != v` detects NaN.
        if v != v:
            return None
        return float(v)

    if algo == "girth":
        # Counterpart of igraph_girth(_, &result, NULL). python-igraph
        # returns float('inf') for acyclic graphs; we encode that as None
        # to match the Rust `Option<u32>` shape.
        v = g.girth()
        if v == float("inf"):
            return None
        return int(v)

    if algo == "is_biconnected":
        # Counterpart of igraph_is_biconnected(_, &result).
        return bool(g.is_biconnected())

    if algo == "bridges":
        # Counterpart of igraph_bridges(_, &result). python-igraph returns
        # a list of edge ids; sort here for stable comparison.
        return sorted(int(e) for e in g.bridges())

    if algo == "articulation_points":
        # Counterpart of igraph_articulation_points(_, &result).
        # python-igraph's `Graph.articulation_points()` returns a list of
        # vertex ids. Order may differ from upstream C — we sort both
        # before comparing in the oracle test (as we do for non-canonical
        # outputs).
        return sorted(int(v) for v in g.articulation_points())

    if algo == "is_eulerian":
        # Counterpart of igraph_is_eulerian(_, &has_path, &has_cycle).
        # python-igraph exposes Graph.is_eulerian(); newer versions return
        # an EulerianResult-like object — fall back to the C-bound helpers.
        try:
            res = g.is_eulerian()
            # python-igraph 0.11 returns 0 / 1 / 2:
            #   0 = neither, 1 = path only, 2 = path + cycle
            if isinstance(res, int):
                return {"has_path": res >= 1, "has_cycle": res == 2}
            return {"has_path": bool(res.has_path), "has_cycle": bool(res.has_cycle)}
        except AttributeError:
            # Very old python-igraph: only is_eulerian_path / cycle helpers.
            return {
                "has_path": bool(g.is_eulerian_path()),
                "has_cycle": bool(g.is_eulerian_cycle()),
            }

    if algo == "distances":
        # Counterpart of igraph_distances(_, NULL, _, single_from, all_to, IGRAPH_OUT).
        # python-igraph returns a list of lists; we ask for a single source so
        # we return the first (and only) row, mapping `inf` -> None to match
        # the Rust `Vec<Option<u32>>` return shape.
        source = int(params["source"])
        row = g.distances(source=source)[0]
        return [None if x == float("inf") else int(x) for x in row]

    if algo == "strongly_connected_components":
        # Counterpart of igraph_connected_components(_, _, _, _, IGRAPH_STRONG).
        # python-igraph returns membership ids in Kosaraju grandfather-pop
        # order (NOT canonicalized to first-seen vertex). The Rust
        # implementation matches the same order line-for-line, so we can
        # compare label vectors directly.
        cc = g.connected_components(mode="strong")
        return {"membership": list(cc.membership), "count": len(cc)}

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
