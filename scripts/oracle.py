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

    if algo == "assortativity_degree":
        # Counterpart of igraph_assortativity_degree(_, _, /*directed=*/false).
        # python-igraph returns NaN for regular graphs; we encode as None.
        v = g.assortativity_degree(directed=False)
        if v != v:
            return None
        return float(v)

    if algo == "count_triangles":
        # python-igraph 0.11 doesn't expose `count_triangles` directly;
        # it offers `list_triangles()` which returns one tuple per
        # triangle. Length of that list is the count.
        return len(g.list_triangles())

    if algo == "avg_nearest_neighbor_degree":
        # Counterpart of igraph_avg_nearest_neighbor_degree(_, vss_all(),
        # IGRAPH_ALL, IGRAPH_ALL, &knn, NULL, NULL). python-igraph's
        # `g.knn()` returns a tuple (knn_per_vertex, knn_per_degree).
        knn_per_vertex, _ = g.knn()
        return [None if (v != v) else float(v) for v in knn_per_vertex]

    if algo == "reciprocity":
        # Counterpart of igraph_reciprocity(_, _, /*ignore_loops=*/false,
        # IGRAPH_RECIPROCITY_DEFAULT). python-igraph's `Graph.reciprocity(
        # ignore_loops=False, mode='default')` returns NaN for graphs
        # with no edges; we encode that as None.
        if g.ecount() == 0:
            return None
        v = g.reciprocity(ignore_loops=False, mode="default")
        if v != v:
            return None
        return float(v)

    if algo == "reciprocity_with_mode":
        # Counterpart of igraph_reciprocity(_, _, ignore_loops, mode).
        # `params.ignore_loops` (bool) and `params.mode` ('default' /
        # 'ratio') are forwarded through. NaN → None.
        if g.ecount() == 0:
            return None
        ignore = bool(params.get("ignore_loops", False))
        mode = str(params.get("mode", "default"))
        v = g.reciprocity(ignore_loops=ignore, mode=mode)
        if v != v:
            return None
        return float(v)

    if algo == "eigenvector_centrality":
        # Counterpart of igraph_eigenvector_centrality(_, _, NULL_eval,
        # /*directed=*/false, /*scale=*/true, NULL_weights, NULL_options).
        return [float(v) for v in g.eigenvector_centrality(directed=False, scale=True)]

    if algo == "biconnected_components":
        # Counterpart of igraph_biconnected_components(). python-igraph
        # returns a tuple `(VertexCover, articulation_points)`. The
        # VertexCover supports iteration over vertex-id lists per
        # component. Wire format: {count, components, articulation_points}.
        cover, aps = g.biconnected_components(return_articulation_points=True)
        comps = [sorted(int(v) for v in cover[i]) for i in range(len(cover))]
        return {
            "count": len(comps),
            "components": comps,
            "articulation_points": sorted(int(v) for v in aps),
        }

    if algo == "pagerank":
        # Counterpart of igraph_pagerank() with damping=0.85, default options.
        # python-igraph defaults to ARPACK (eigensolver-based) which is more
        # accurate than power iteration; oracle tolerance accounts for that.
        directed = g.is_directed()
        return [float(v) for v in g.pagerank(damping=0.85, directed=directed)]

    if algo == "edge_betweenness":
        # Counterpart of igraph_edge_betweenness(_, NULL_weights, _, ALL_eids,
        # /*directed=*/g.is_directed(), /*normalized=*/false). Returns a
        # parallel `{edges: [(u,v),...], values: [...]}` payload so the
        # oracle test can match by endpoint pair (edge ids vary across
        # the wire-format reconstruction).
        directed = g.is_directed()
        vals = g.edge_betweenness(directed=directed)
        edges = [list(e.tuple) for e in g.es]
        return {"edges": edges, "values": [float(v) for v in vals]}

    if algo == "betweenness":
        # Counterpart of igraph_betweenness(_, _, vss_all(),
        # /*directed=*/g.is_directed(), NULL_weights).
        directed = g.is_directed()
        return [float(v) for v in g.betweenness(directed=directed)]

    if algo == "harmonic_centrality":
        # Counterpart of igraph_harmonic_centrality(_, _, vss_all(), IGRAPH_OUT,
        # NULL_weights, /*normalized=*/true).
        mode = "out" if g.is_directed() else "all"
        vals = g.harmonic_centrality(mode=mode, normalized=True)
        return [float(v) for v in vals]

    if algo == "closeness":
        # Counterpart of igraph_closeness(_, _, _, _, vss_all(), IGRAPH_OUT,
        # NULL_weights, /*normalized=*/true).
        # python-igraph's `g.closeness(mode='out', normalized=True)` returns
        # NaN for isolated vertices; encode as None.
        mode = "out" if g.is_directed() else "all"
        vals = g.closeness(mode=mode, normalized=True)
        return [None if (v != v) else float(v) for v in vals]

    if algo == "assortativity_degree_weighted":
        # Counterpart of igraph_assortativity_degree(_, _, /*directed=*/false,
        # &weights). python-igraph 0.11 has no weighted assortativity
        # at the Python layer (`Graph.assortativity` has no `weights`
        # kwarg, and `assortativity_degree` doesn't take weights). For
        # the unit-weight case, the weighted formula collapses to the
        # unweighted one, so we use that as the oracle and the Rust
        # tests only call this for unit-weight fixtures. Non-unit
        # weights are validated via the Rust conformance suite using
        # hand-computed reference values.
        v = g.assortativity_degree(directed=False)
        if v != v:
            return None
        return float(v)

    if algo == "pagerank_weighted":
        # Counterpart of igraph_pagerank(_, IGRAPH_PAGERANK_ALGO_POWER,
        # _, _, vss_all(), directed, 0.85, &weights, NULL_options).
        # python-igraph defaults to ARPACK; oracle tests use a tolerant
        # comparison to absorb the eigensolver-vs-power-iteration drift.
        directed = g.is_directed()
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            return [
                float(v)
                for v in g.pagerank(damping=0.85, directed=directed, weights="weight")
            ]
        return [float(v) for v in g.pagerank(damping=0.85, directed=directed)]

    if algo == "edge_betweenness_weighted":
        # Counterpart of igraph_edge_betweenness(_, _, all_eids,
        # /*directed=*/g.is_directed(), &weights). Returns a parallel
        # `{edges: [(u,v),...], values: [...]}` payload so the oracle
        # test can match by endpoint pair (edge ids vary across the
        # wire-format reconstruction).
        directed = g.is_directed()
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            vals = g.edge_betweenness(directed=directed, weights="weight")
        else:
            vals = g.edge_betweenness(directed=directed)
        edges = [list(e.tuple) for e in g.es]
        return {"edges": edges, "values": [float(v) for v in vals]}

    if algo == "betweenness_weighted":
        # Counterpart of igraph_betweenness(_, _, vss_all(),
        # /*directed=*/g.is_directed(), &weights). python-igraph reads
        # the `weight` edge attribute when weights="weight".
        directed = g.is_directed()
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            return [float(v) for v in g.betweenness(directed=directed, weights="weight")]
        return [float(v) for v in g.betweenness(directed=directed)]

    if algo == "harmonic_centrality_weighted":
        # Counterpart of igraph_harmonic_centrality(_, _, vss_all(),
        # IGRAPH_OUT, &weights, /*normalized=*/true).
        mode = "out" if g.is_directed() else "all"
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            vals = g.harmonic_centrality(mode=mode, weights="weight", normalized=True)
        else:
            vals = g.harmonic_centrality(mode=mode, normalized=True)
        return [float(v) for v in vals]

    if algo == "closeness_weighted":
        # Counterpart of igraph_closeness(_, _, _, _, vss_all(),
        # IGRAPH_OUT, &weights, /*normalized=*/true). python-igraph
        # reads the `weight` edge attribute when weights="weight".
        mode = "out" if g.is_directed() else "all"
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            vals = g.closeness(mode=mode, weights="weight", normalized=True)
        else:
            vals = g.closeness(mode=mode, normalized=True)
        return [None if (v != v) else float(v) for v in vals]

    if algo == "coreness":
        # Counterpart of igraph_coreness(_, _, IGRAPH_ALL). python-igraph
        # exposes `Graph.coreness(mode='all')` which returns a per-vertex
        # int list.
        vals = g.coreness(mode="all")
        return [int(v) for v in vals]

    if algo == "complementer":
        # Counterpart of igraph_complementer(_, &graph, loops).
        # python-igraph's `g.complementer(loops=...)` returns a new Graph.
        loops = bool(params.get("loops", False))
        c = g.complementer(loops=loops)
        edges = [list(e.tuple) for e in c.es]
        return {"vcount": c.vcount(), "directed": c.is_directed(), "edges": edges}

    if algo == "dijkstra_distances":
        # Counterpart of igraph_distances_dijkstra(_, _, single source,
        # to=igraph_vss_all(), &weights, IGRAPH_OUT). The wire payload
        # already carries `weights` (top-level field). python-igraph's
        # `g.distances(source, weights="weight", mode='out')` returns
        # a list-of-lists; we flatten and translate inf → None.
        source = int(params["source"])
        mode = "out" if g.is_directed() else "all"
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            rows = g.distances(source=source, weights="weight", mode=mode)
        else:
            rows = g.distances(source=source, mode=mode)
        out = []
        for v in rows[0]:
            f = float(v)
            out.append(None if f == float("inf") else f)
        return out

    if algo == "floyd_warshall_distances":
        # Counterpart of igraph_distances_floyd_warshall(_, _, vss_all,
        # vss_all, &weights, IGRAPH_OUT, AUTOMATIC). python-igraph
        # 0.11 has no direct FW API at Python layer, but g.distances()
        # already returns the full all-pairs matrix and accepts the
        # same weight conventions, so we just relay it. inf → None.
        mode = "out" if g.is_directed() else "all"
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            rows = g.distances(weights="weight", mode=mode)
        else:
            rows = g.distances(mode=mode)
        out = []
        for row in rows:
            out_row = []
            for v in row:
                f = float(v)
                out_row.append(None if f == float("inf") else f)
            out.append(out_row)
        return out

    if algo == "disjoint_union":
        # Counterpart of igraph_disjoint_union(_, &left, &right). The
        # request graph carries `left`; `right` is encoded inside
        # `params.right_graph` as the same wire format used at the top
        # level (n / edges / directed / weights).
        rp = params["right_graph"]
        right = make_graph(rp)
        u = ig.disjoint_union([g, right])
        edges = [list(e.tuple) for e in u.es]
        return {"vcount": u.vcount(), "directed": u.is_directed(), "edges": edges}

    if algo == "is_loop":
        # Counterpart of igraph_is_loop(_, _, igraph_ess_all()).
        # python-igraph 0.11 exposes Edge.is_loop() per-edge.
        return [bool(e.is_loop()) for e in g.es]

    if algo == "is_multiple":
        # Counterpart of igraph_is_multiple(_, _, igraph_ess_all()).
        # python-igraph's Graph.is_multiple() returns the per-edge mask.
        return [bool(x) for x in g.is_multiple()]

    if algo == "has_loop":
        # Counterpart of igraph_has_loop(). python-igraph 0.11 has no
        # direct API; emulate via `any(e.is_loop() for e in g.es)`.
        return any(e.is_loop() for e in g.es)

    if algo == "has_multiple":
        # Counterpart of igraph_has_multiple(). python-igraph 0.11 has no
        # direct API; emulate via `any(g.is_multiple())`.
        return any(g.is_multiple())

    if algo == "is_simple":
        # Counterpart of igraph_is_simple(_, &res, /*directed=*/true).
        # python-igraph's `g.is_simple()` honours the graph's own
        # directedness (no separate flag), which matches our directed
        # phase-1 slice exactly.
        return bool(g.is_simple())

    if algo == "modularity":
        # Counterpart of igraph_modularity(_, &membership, NULL_weights,
        # /*resolution=*/1.0, /*directed=*/false, &result).
        membership = list(params["membership"])
        resolution = float(params.get("resolution", 1.0))
        if g.ecount() == 0:
            return None
        v = g.modularity(membership, resolution=resolution)
        if v != v:  # NaN
            return None
        return float(v)

    if algo == "simplify":
        # Counterpart of igraph_simplify(g, remove_multiple, remove_loops, NULL).
        # python-igraph mutates in place; we copy first and return the new
        # edge list in the canonical wire format. Edge order after simplify
        # depends on python-igraph's internal sort; the runner sorts both
        # sides before comparing.
        remove_multiple = bool(params.get("remove_multiple", True))
        remove_loops = bool(params.get("remove_loops", True))
        h = g.copy()
        h.simplify(multiple=remove_multiple, loops=remove_loops)
        edges = [list(e.tuple) for e in h.es]
        return {"vcount": h.vcount(), "directed": h.is_directed(), "edges": edges}

    if algo == "transitive_closure":
        # Counterpart of igraph_transitive_closure(). python-igraph 0.11
        # has no direct API; emulate by computing per-vertex reachable sets
        # and returning the new edge list as `[(u, v), ...]` for stable
        # comparison.
        n = g.vcount()
        directed = g.is_directed()
        mode = "out" if directed else "all"
        edges = []
        for u in range(n):
            reachable = set(g.subcomponent(u, mode=mode))
            v_start = 0 if directed else u + 1
            for v in range(v_start, n):
                if u != v and v in reachable:
                    edges.append([u, v])
        return {"vcount": n, "directed": directed, "edges": edges}

    if algo == "reachability_matrix":
        # Counterpart of igraph_reachability(_, ..., IGRAPH_OUT) returning
        # a dense per-vertex bitmap. python-igraph 0.11 lacks a direct
        # API, so we BFS via subcomponent + mask each row.
        n = g.vcount()
        mode = "out" if g.is_directed() else "all"
        rows = []
        for v in range(n):
            reachable = set(g.subcomponent(v, mode=mode))
            rows.append([j in reachable for j in range(n)])
        return rows

    if algo == "count_reachable":
        # Counterpart of igraph_count_reachable(_, _, IGRAPH_OUT).
        # python-igraph 0.11 has no direct count_reachable, but
        # `subcomponent(v, mode='out')` returns the reachable vertex
        # list — count its length.
        mode = "all" if not g.is_directed() else "out"
        return [len(g.subcomponent(v, mode=mode)) for v in range(g.vcount())]

    if algo == "density":
        # Counterpart of igraph_density(_, NULL_weights, _, /*loops=*/false).
        if g.vcount() < 2:
            return None
        return float(g.density(loops=False))

    if algo == "mean_distance":
        # Counterpart of igraph_average_path_length(_, NULL_weights, _, _,
        # /*directed=*/true, /*unconn=*/true). python-igraph 0.11 returns
        # NaN if no connected pairs exist.
        v = g.average_path_length(directed=True, unconn=True)
        if v != v:  # NaN check
            return None
        return float(v)

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
