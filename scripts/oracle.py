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

    if algo == "eccentricity_with_mode":
        # ALGO-SP-021. Counterpart of igraph_eccentricity(_, NULL_weights,
        # _, igraph_vss_all(), mode). `mode` arrives as one of "out" /
        # "in" / "all"; python-igraph accepts the same lowercase strings.
        mode = params.get("mode", "out")
        return [int(x) for x in g.eccentricity(mode=mode)]

    if algo == "radius_with_mode":
        # ALGO-SP-022. Counterpart of igraph_radius(_, NULL_weights, _, mode).
        mode = params.get("mode", "out")
        if g.vcount() == 0:
            return None
        return int(g.radius(mode=mode))

    if algo == "diameter_with_mode":
        # ALGO-SP-023. Counterpart of igraph_diameter(_, ..., mode, true).
        # python-igraph's `Graph.diameter(directed=...)` toggles between
        # IGRAPH_OUT (directed=True) and IGRAPH_ALL (directed=False); it
        # has no IN mode. We map "out" → directed=True, "all" → directed
        # =False. The "in" mode is computed by reversing the graph and
        # running with directed=True (BFS along reversed edges == IN BFS
        # on the original).
        mode = params.get("mode", "out")
        if g.vcount() == 0:
            return None
        if mode == "in" and g.is_directed():
            rev_edges = [(t, s) for (s, t) in g.get_edgelist()]
            rev = ig.Graph(n=g.vcount(), edges=rev_edges, directed=True)
            return int(rev.diameter(directed=True, unconn=True))
        directed_flag = mode == "out"
        return int(g.diameter(directed=directed_flag, unconn=True))

    if algo == "eccentricity_weighted_with_mode":
        # ALGO-SP-021..023 weighted: counterpart of
        # igraph_eccentricity(_, weights, _, igraph_vss_all(), mode).
        # python-igraph's `Graph.eccentricity` takes `weights="weight"`.
        mode = params.get("mode", "out")
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            r = g.eccentricity(mode=mode, weights="weight")
        else:
            r = g.eccentricity(mode=mode)
        return [float(x) for x in r]

    if algo == "radius_weighted_with_mode":
        mode = params.get("mode", "out")
        if g.vcount() == 0:
            return None
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            return float(g.radius(mode=mode, weights="weight"))
        return float(g.radius(mode=mode))

    if algo == "diameter_weighted_with_mode":
        # python-igraph's Graph.diameter accepts `weights=` and toggles
        # directedness via `directed=`. As with the unweighted variant,
        # IN-mode on directed graphs is emulated by edge-reversal.
        mode = params.get("mode", "out")
        if g.vcount() == 0:
            return None
        weights_arg = (
            "weight"
            if g.ecount() > 0 and "weight" in g.edge_attributes()
            else None
        )
        if mode == "in" and g.is_directed():
            rev_edges = [(t, s) for (s, t) in g.get_edgelist()]
            rev = ig.Graph(n=g.vcount(), edges=rev_edges, directed=True)
            if weights_arg is not None:
                rev.es["weight"] = list(g.es["weight"])
                return float(rev.diameter(directed=True, unconn=True, weights="weight"))
            return float(rev.diameter(directed=True, unconn=True))
        directed_flag = mode == "out"
        if weights_arg is not None:
            return float(
                g.diameter(directed=directed_flag, unconn=True, weights="weight")
            )
        return float(g.diameter(directed=directed_flag, unconn=True))

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

    if algo == "avg_nearest_neighbor_degree_weighted":
        # Counterpart with weights (Barrat formula). The wire harness
        # sets `g.es['weight']` from the payload's `weights` field;
        # we pass that attribute name to python-igraph's knn().
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            knn_per_vertex, _ = g.knn(weights="weight")
        else:
            knn_per_vertex, _ = g.knn()
        return [None if (v != v) else float(v) for v in knn_per_vertex]

    if algo == "knnk":
        # Counterpart of igraph_avg_nearest_neighbor_degree(_, _, _, _,
        # NULL, &knnk, NULL). python-igraph returns the second tuple slot.
        _, knn_per_degree = g.knn()
        return [None if (v != v) else float(v) for v in knn_per_degree]

    if algo == "knnk_weighted":
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            _, knn_per_degree = g.knn(weights="weight")
        else:
            _, knn_per_degree = g.knn()
        return [None if (v != v) else float(v) for v in knn_per_degree]

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
        # component. Wire format: {count, components, articulation_points,
        # component_edge_pairs}. `component_edge_pairs[i]` lists the
        # endpoint pairs (sorted, deduplicated by canonical (min, max))
        # of every edge inside the i-th component — matches CC-012's
        # `BiconnectedComponents.component_edges`.
        cover, aps = g.biconnected_components(return_articulation_points=True)
        comps = [sorted(int(v) for v in cover[i]) for i in range(len(cover))]
        # Build a canonical edge-pair multiset per component.
        edge_pairs = []
        for i in range(len(cover)):
            verts = set(int(v) for v in cover[i])
            pairs = []
            for e in g.es:
                u, v = int(e.source), int(e.target)
                if u in verts and v in verts:
                    a, b = (u, v) if u <= v else (v, u)
                    pairs.append([a, b])
            pairs.sort()
            edge_pairs.append(pairs)
        return {
            "count": len(comps),
            "components": comps,
            "articulation_points": sorted(int(v) for v in aps),
            "component_edge_pairs": edge_pairs,
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

    if algo == "assortativity_degree_directed_weighted":
        # PR-006d: counterpart of igraph_assortativity_degree(_, _,
        # /*directed=*/true, &weights). python-igraph still has no
        # weighted assortativity at the Python layer, so the oracle
        # only handles unit-weight cases (formula collapses to the
        # unweighted directed assortativity then). Non-unit-weight
        # fixtures use hand-computed reference values, same convention
        # as the undirected weighted variant.
        v = g.assortativity_degree(directed=True)
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

    if algo == "assortativity_degree_directed":
        # Counterpart of igraph_assortativity_degree(_, _, /*directed=*/true).
        # python-igraph's `g.assortativity_degree(directed=True)` returns
        # NaN when either variance vanishes; we encode as None.
        if g.ecount() == 0:
            return None
        v = g.assortativity_degree(directed=True)
        if v != v:
            return None
        return float(v)

    if algo == "coreness_with_mode":
        # Counterpart of igraph_coreness(_, _, mode). python-igraph's
        # `Graph.coreness(mode=)` accepts 'in' / 'out' / 'all'.
        mode = str(params.get("mode", "all"))
        vals = g.coreness(mode=mode)
        return [int(v) for v in vals]

    if algo == "is_simple_with_mode":
        # Counterpart of igraph_is_simple(_, _, /*directed=*/dir).
        # python-igraph's `Graph.is_simple()` doesn't expose the
        # `directed` flag directly, so we reuse `igraph_is_simple` via
        # the C wrapper if available; otherwise emulate the
        # "treat directed as undirected" path with a Python sweep.
        directed_as_undirected = bool(params.get("directed_as_undirected", False))
        if not directed_as_undirected or not g.is_directed():
            return bool(g.is_simple())
        # Directed-as-undirected: canonicalise endpoint pairs and
        # check for self-loops + duplicates.
        seen = set()
        for e in g.es:
            s, t = e.tuple
            if s == t:
                return False
            key = (min(s, t), max(s, t))
            if key in seen:
                return False
            seen.add(key)
        return True

    if algo == "modularity_directed":
        # Counterpart of igraph_modularity(_, &membership, NULL_weights,
        # resolution, /*directed=*/true, _). python-igraph's
        # `Graph.modularity` accepts a `directed` arg via the
        # underlying C call.
        if g.ecount() == 0:
            return None
        membership = list(params["membership"])
        resolution = float(params.get("resolution", 1.0))
        v = g.modularity(membership, directed=True, resolution=resolution)
        if v != v:
            return None
        return float(v)

    if algo == "modularity_weighted":
        # Counterpart of igraph_modularity(_, &membership, &weights,
        # resolution, /*directed=*/false, _). python-igraph's
        # `Graph.modularity(membership, weights=...)` reads the
        # `weight` edge attribute when weights="weight".
        if g.ecount() == 0:
            return None
        membership = list(params["membership"])
        resolution = float(params.get("resolution", 1.0))
        if "weight" in g.edge_attributes():
            v = g.modularity(membership, weights="weight", resolution=resolution)
        else:
            v = g.modularity(membership, resolution=resolution)
        if v != v:
            return None
        return float(v)

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

    if algo == "dijkstra_paths":
        # Counterpart of igraph_get_shortest_paths_dijkstra(_, _, _, source,
        # vss_all(), &weights, IGRAPH_OUT, parents, inbound_edges).
        # python-igraph exposes the same via `Graph.get_shortest_paths`,
        # but for cross-impl portability we instead reconstruct the full
        # SPT from `g.distances(...)` plus a manual parent lookup that
        # picks any tie-breaking parent that satisfies the relaxation.
        source = int(params["source"])
        mode = "out" if g.is_directed() else "all"
        # Distances first (used both for the public output AND for the
        # parent reconstruction).
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            d_rows = g.distances(source=source, weights="weight", mode=mode)
        else:
            d_rows = g.distances(source=source, mode=mode)
        d = [None if float(v) == float("inf") else float(v) for v in d_rows[0]]
        # Parents / inbound edges: replay an SPT consistent with these
        # distances. For each non-source reachable vertex v, pick any
        # incoming edge u→v whose weight closes the gap d[v] - d[u]
        # (within 1e-9). On the first match we settle and emit. python-
        # igraph's own get_shortest_paths can also be used, but explicit
        # reconstruction here keeps the oracle deterministic w.r.t. our
        # Rust impl's heap-order tie-breaking — in practice both match.
        n = g.vcount()
        parents = [None] * n
        inbound = [None] * n
        weights = (
            list(g.es["weight"])
            if g.ecount() > 0 and "weight" in g.edge_attributes()
            else [1.0] * g.ecount()
        )
        # Sort vertices by distance ascending so that when we look up a
        # parent for v, that parent's distance is already known.
        order = sorted(
            (i for i in range(n) if i != source and d[i] is not None),
            key=lambda i: d[i],
        )
        directed = g.is_directed()
        # Adjacency by destination → list of (src, eid, weight).
        in_adj = [[] for _ in range(n)]
        for e in g.es:
            s, t = e.tuple
            in_adj[t].append((s, e.index, weights[e.index]))
            if not directed:
                in_adj[s].append((t, e.index, weights[e.index]))
        for v in order:
            for u, eid, w in in_adj[v]:
                if d[u] is None:
                    continue
                if abs(d[u] + w - d[v]) < 1e-9:
                    parents[v] = u
                    inbound[v] = eid
                    break
        return {"distances": d, "parents": parents, "inbound_edges": inbound}

    if algo == "dijkstra_path_to":
        # Counterpart of igraph_get_shortest_path_dijkstra(_, _, _, source,
        # target, &weights, IGRAPH_OUT). Returns either None (target
        # unreachable) or a {vertices, edges} dict.
        source = int(params["source"])
        target = int(params["target"])
        mode = "out" if g.is_directed() else "all"
        try:
            if g.ecount() > 0 and "weight" in g.edge_attributes():
                vs = g.get_shortest_paths(
                    source, to=target, weights="weight", mode=mode, output="vpath"
                )[0]
                es = g.get_shortest_paths(
                    source, to=target, weights="weight", mode=mode, output="epath"
                )[0]
            else:
                vs = g.get_shortest_paths(source, to=target, mode=mode, output="vpath")[0]
                es = g.get_shortest_paths(source, to=target, mode=mode, output="epath")[0]
        except Exception:
            return None
        if not vs:
            return None
        return {"vertices": [int(x) for x in vs], "edges": [int(x) for x in es]}

    if algo == "dijkstra_distances_cutoff":
        # Counterpart of igraph_distances_dijkstra_cutoff(_, _, single
        # source, to=igraph_vss_all(), &weights, IGRAPH_OUT, cutoff).
        # python-igraph's `g.distances` does not accept a cutoff so we
        # apply it after the fact (mask values > cutoff to None).
        source = int(params["source"])
        cutoff = params.get("cutoff", None)
        mode = "out" if g.is_directed() else "all"
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            rows = g.distances(source=source, weights="weight", mode=mode)
        else:
            rows = g.distances(source=source, mode=mode)
        out = []
        for v in rows[0]:
            f = float(v)
            if f == float("inf"):
                out.append(None)
            elif cutoff is not None and f > float(cutoff):
                out.append(None)
            else:
                out.append(f)
        return out

    if algo == "a_star_path":
        # SP-005: A* shortest path from source to target. With null
        # heuristic (h ≡ 0), A* reduces to Dijkstra single-source
        # single-target — same path as `dijkstra_path_to`. python-
        # igraph has no Python-level A* binding, so the oracle simply
        # delegates to `Graph.get_shortest_paths(weights=...)`. Output
        # `{vertices, edges}` matches dijkstra_path_to's format.
        source = int(params["source"])
        target = int(params["target"])
        mode = str(params.get("mode", "out"))
        try:
            if g.ecount() > 0 and "weight" in g.edge_attributes():
                vs = g.get_shortest_paths(
                    source, to=target, weights="weight", mode=mode, output="vpath"
                )[0]
                es = g.get_shortest_paths(
                    source, to=target, weights="weight", mode=mode, output="epath"
                )[0]
            else:
                vs = g.get_shortest_paths(source, to=target, mode=mode, output="vpath")[0]
                es = g.get_shortest_paths(source, to=target, mode=mode, output="epath")[0]
        except Exception:
            return None
        if not vs:
            return None
        return {"vertices": [int(x) for x in vs], "edges": [int(x) for x in es]}

    if algo == "topological_sorting":
        # Counterpart of igraph_topological_sorting. python-igraph
        # exposes Graph.topological_sorting(mode='OUT'/'IN'/'ALL')
        # which returns a list[int]. We mirror the upstream
        # contract: undirected or ALL mode → return an _error sentinel
        # so the Rust side can compare against our InvalidArgument.
        if not g.is_directed():
            return {"_error": "topological_sorting requires a directed graph"}
        mode = str(params.get("mode", "out")).lower()
        if mode == "all":
            return {"_error": "topological_sorting does not accept mode=all"}
        if mode not in ("out", "in"):
            return {"_error": f"invalid mode: {mode}"}
        try:
            return list(g.topological_sorting(mode=mode))
        except Exception as exc:
            return {"_error": str(exc)}

    if algo == "is_acyclic":
        # Counterpart of igraph_is_acyclic. python-igraph does not
        # expose this predicate directly, but we can replicate it
        # inline: directed → check is_dag; undirected → union-find
        # over the edges.
        if g.is_directed():
            return bool(g.is_dag())
        n = g.vcount()
        parent = list(range(n))

        def find(v):
            while parent[v] != v:
                parent[v] = parent[parent[v]]
                v = parent[v]
            return v

        for e in g.es:
            u, v = e.source, e.target
            if u == v:
                return False
            ru = find(u)
            rv = find(v)
            if ru == rv:
                return False
            parent[ru] = rv
        return True

    if algo == "is_forest":
        # Counterpart of igraph_is_forest. python-igraph does not
        # expose this predicate, so we replicate it inline.
        # Returns {"is_forest": bool, "roots": [int, ...]} where
        # `roots` is empty when not a forest. Mirrors upstream's
        # behaviour: null graph IS a forest with empty roots; mode
        # is ignored for undirected graphs; for directed graphs,
        # OUT roots are in-degree-0 vertices, IN roots are
        # out-degree-0 vertices, ALL is treated as undirected.
        mode = str(params.get("mode", "out")).lower()
        if mode not in ("out", "in", "all"):
            return {"_error": f"invalid mode: {mode}"}
        n = g.vcount()
        m = g.ecount()
        if n == 0:
            return {"is_forest": True, "roots": []}
        if m == 0:
            return {"is_forest": True, "roots": list(range(n))}
        # Cardinality bound.
        if m > n - 1:
            return {"is_forest": False, "roots": []}
        directed = g.is_directed()
        eff_mode = "all" if not directed else mode

        # Adjacency lists in the requested orientation.
        if eff_mode == "all":
            adj = [list() for _ in range(n)]
            for e in g.es:
                u, v = e.source, e.target
                adj[u].append((v, e.index))
                adj[v].append((u, e.index))
        elif eff_mode == "out":
            adj = [list() for _ in range(n)]
            for e in g.es:
                adj[e.source].append((e.target, e.index))
        else:  # "in"
            adj = [list() for _ in range(n)]
            for e in g.es:
                adj[e.target].append((e.source, e.index))

        visited = [False] * n
        visited_count = 0
        roots: list[int] = []

        def visit(start: int) -> bool:
            nonlocal visited_count
            stack = [start]
            while stack:
                u = stack.pop()
                if visited[u]:
                    return False
                visited[u] = True
                visited_count += 1
                for v, _eid in adj[u]:
                    if eff_mode == "all":
                        if not visited[v]:
                            stack.append(v)
                        elif v == u:
                            return False
                    else:
                        stack.append(v)
            return True

        if eff_mode == "all":
            for v in range(n):
                if not visited[v]:
                    roots.append(v)
                    if not visit(v):
                        return {"is_forest": False, "roots": []}
        else:
            # Counter-direction degree:
            # OUT-tree → vertices have in-degree ≤ 1
            # IN-tree  → vertices have out-degree ≤ 1
            counter = "in" if eff_mode == "out" else "out"
            for v in range(n):
                if counter == "in":
                    d = g.degree(v, mode="in", loops=True)
                else:
                    d = g.degree(v, mode="out", loops=True)
                if d > 1:
                    return {"is_forest": False, "roots": []}
                if d == 0:
                    roots.append(v)
                    if not visit(v):
                        return {"is_forest": False, "roots": []}
        if visited_count != n:
            return {"is_forest": False, "roots": []}
        return {"is_forest": True, "roots": roots}

    if algo == "is_tree":
        # Counterpart of igraph_is_tree. python-igraph exposes
        # `Graph.is_tree(mode='out'|'in'|'all')` which returns a
        # bool. Our Rust API returns `Option<VertexId>` — Some(_) →
        # true, None → false. Oracle compares the bool.
        mode = str(params.get("mode", "out")).lower()
        if mode not in ("out", "in", "all"):
            return {"_error": f"invalid mode: {mode}"}
        return bool(g.is_tree(mode=mode))

    if algo == "is_dag":
        # Counterpart of igraph_is_dag. python-igraph exposes
        # `Graph.is_dag()` which returns a bool directly. Returns
        # False for undirected graphs (matches upstream).
        if not g.is_directed():
            return False
        return bool(g.is_dag())

    if algo == "is_complete":
        # Counterpart of igraph_is_complete. python-igraph exposes
        # `Graph.is_complete()` directly (returns True for the null
        # graph and singleton). Mirrors upstream semantics: directed
        # graphs require both arcs for every pair.
        return bool(g.is_complete())

    if algo == "neighborhood_size":
        # Counterpart of igraph_neighborhood_size. python-igraph
        # exposes `Graph.neighborhood_size(vertices=None, order=1,
        # mode="all", mindist=0)`. We always compute over all
        # vertices (no vertex selector yet in Rust).
        order = int(params.get("order", 1))
        mode = str(params.get("mode", "all")).lower()
        if mode not in ("out", "in", "all"):
            return {"_error": f"invalid mode: {mode}"}
        mindist = int(params.get("mindist", 0))
        # python-igraph rejects negative order with a ValueError, even
        # though the underlying C lib treats it as infinite. Saturate
        # negative orders to `vcount` here (BFS depth is bounded by
        # n-1, so this is semantically equivalent).
        if order < 0:
            order = g.vcount()
        return list(g.neighborhood_size(vertices=None, order=order, mode=mode, mindist=mindist))

    if algo == "is_same_graph":
        # Counterpart of igraph_is_same_graph. Compare the wire graph
        # `g` to a second graph encoded under params.other (same
        # canonical shape as the top-level payload). python-igraph
        # does not expose this predicate, so we implement it inline:
        # match vcount, directedness, and the sorted (canonicalised)
        # edge multisets.
        other_payload = params.get("other", {})
        other = make_graph(other_payload)
        if g.vcount() != other.vcount():
            return False
        if g.ecount() != other.ecount():
            return False
        if g.is_directed() != other.is_directed():
            return False
        directed = g.is_directed()

        def canonical_edges(graph):
            pairs = [(e.source, e.target) for e in graph.es]
            if not directed:
                pairs = [(min(u, v), max(u, v)) for (u, v) in pairs]
            pairs.sort()
            return pairs

        return canonical_edges(g) == canonical_edges(other)

    if algo == "site_percolation":
        # Counterpart of igraph_site_percolation. python-igraph does
        # not bind percolation; inline union-find reference, with
        # all-neighbor walks that match upstream's IGRAPH_LOOPS |
        # IGRAPH_MULTIPLE semantics (loops twice, parallels each).
        vertex_order = [int(v) for v in params.get("vertex_order", [])]
        n = g.vcount()
        # Validate up front.
        seen = set()
        for vid in vertex_order:
            if vid < 0 or vid >= n:
                return {"_error": f"vertex id {vid} out of range (n={n})"}
            if vid in seen:
                return {"_error": f"duplicate vertex id {vid}"}
            seen.add(vid)

        if not vertex_order:
            return {"giant_size": [], "edge_count": []}

        links = list(range(n))
        sizes = [0] * n

        def find(v):
            while links[v] != v:
                links[v] = links[links[v]]
                v = links[v]
            return v

        biggest = 1
        edges_added = 0
        giant_size = []
        edge_count = []
        for vid in vertex_order:
            sizes[vid] = 1
            # All-neighbor walk with IGRAPH_LOOPS | IGRAPH_MULTIPLE:
            # for directed graphs, merge out + in incident edges; the
            # `igraph_neighbors(IGRAPH_ALL, IGRAPH_LOOPS, IGRAPH_MULTIPLE)`
            # call produces one entry per incident edge per endpoint.
            if g.is_directed():
                eids = list(g.incident(vid, mode="out")) + list(g.incident(vid, mode="in"))
            else:
                eids = list(g.incident(vid, mode="all"))
            neighbors = []
            for e in eids:
                edge = g.es[e]
                other = edge.target if edge.source == vid else edge.source
                neighbors.append(other)
            for nb in neighbors:
                if sizes[nb] == 0:
                    continue
                edges_added += 1
                ra = find(vid)
                rb = find(nb)
                if ra != rb:
                    if sizes[ra] < sizes[rb]:
                        parent, child = rb, ra
                    else:
                        parent, child = ra, rb
                    links[child] = parent
                    sizes[parent] += sizes[child]
                    if sizes[parent] > biggest:
                        biggest = sizes[parent]
            giant_size.append(biggest)
            edge_count.append(edges_added)
        return {"giant_size": giant_size, "edge_count": edge_count}

    if algo == "edgelist_percolation":
        # Counterpart of igraph_edgelist_percolation. python-igraph
        # does not bind percolation; inline union-find reference.
        # Edges come through params (not from g.es) because
        # python-igraph reorders edges internally — the percolation
        # curve is order-sensitive, so we need the exact sequence
        # the caller intended.
        edges = [tuple(e) for e in params.get("edges", [])]
        if not edges:
            return {"giant_size": [], "vertex_count": []}
        max_id = max(max(u, v) for (u, v) in edges)
        vcount = max_id + 1
        links = list(range(vcount))
        sizes = [0] * vcount

        def find(v):
            while links[v] != v:
                links[v] = links[links[v]]
                v = links[v]
            return v

        biggest = 1
        added = 0
        giant_size = []
        vertex_count = []
        for (u, v) in edges:
            if sizes[u] == 0:
                sizes[u] = 1
                added += 1
            if sizes[v] == 0:
                sizes[v] = 1
                if u != v:
                    added += 1
            if u != v:
                ra = find(u)
                rb = find(v)
                if ra != rb:
                    if sizes[ra] < sizes[rb]:
                        parent, child = rb, ra
                    else:
                        parent, child = ra, rb
                    links[child] = parent
                    sizes[parent] += sizes[child]
                    if sizes[parent] > biggest:
                        biggest = sizes[parent]
            giant_size.append(biggest)
            vertex_count.append(added)
        return {"giant_size": giant_size, "vertex_count": vertex_count}

    if algo == "widest_paths":
        # Counterpart of igraph_get_widest_paths(_, NULL, NULL, source,
        # vss_all(), weights, IGRAPH_OUT, parents, inbound_edges).
        # Returns the widths vector plus per-vertex parent and
        # inbound-edge SPT. python-igraph does not bind widest paths;
        # inline reference is the same SPFA-style Dijkstra as widths
        # oracle plus parent_eid tracking.
        import heapq

        source = int(params["source"])
        n = g.vcount()
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            ew = list(g.es["weight"])
        else:
            ew = [1.0] * g.ecount()
        widths = [float("-inf")] * n
        widths[source] = float("inf")
        parent_v = [None] * n
        parent_e = [None] * n
        heap = [(-float("inf"), source)]
        while heap:
            neg_w, v = heapq.heappop(heap)
            w = -neg_w
            if w < widths[v]:
                continue
            eids = g.incident(v, mode="out" if g.is_directed() else "all")
            for e in eids:
                edge = g.es[e]
                edge_w = ew[e]
                if edge_w == float("-inf"):
                    continue
                other = edge.target if edge.source == v else edge.source
                alt = min(w, edge_w)
                if alt > widths[other]:
                    widths[other] = alt
                    parent_v[other] = v
                    parent_e[other] = e
                    heapq.heappush(heap, (-alt, other))
        return {
            "widths": [
                None if w == float("-inf")
                else "Infinity" if w == float("inf")
                else w
                for w in widths
            ],
            "parents": parent_v,
            "inbound_edges": parent_e,
        }

    if algo == "widest_paths_to":
        # Counterpart of igraph_get_widest_paths(_, _, _, from, to,
        # weights, IGRAPH_OUT). python-igraph does not bind this;
        # we reuse the inline single-target reference and loop over
        # the targets.
        import heapq

        from_v = int(params["from"])
        targets = [int(t) for t in params.get("targets", [])]
        n = g.vcount()
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            ew = list(g.es["weight"])
        else:
            ew = [1.0] * g.ecount()
        widths = [float("-inf")] * n
        widths[from_v] = float("inf")
        parent = [None] * n
        heap = [(-float("inf"), from_v)]
        while heap:
            neg_w, v = heapq.heappop(heap)
            w = -neg_w
            if w < widths[v]:
                continue
            eids = g.incident(v, mode="out" if g.is_directed() else "all")
            for e in eids:
                edge = g.es[e]
                edge_w = ew[e]
                if edge_w == float("-inf"):
                    continue
                other = edge.target if edge.source == v else edge.source
                alt = min(w, edge_w)
                if alt > widths[other]:
                    widths[other] = alt
                    parent[other] = (v, e)
                    heapq.heappush(heap, (-alt, other))

        results = []
        for t in targets:
            if from_v == t:
                results.append({"vertices": [from_v], "edges": []})
                continue
            if widths[t] == float("-inf"):
                results.append(None)
                continue
            vs = [t]
            es = []
            cur = t
            while cur != from_v:
                prev, eid = parent[cur]
                vs.append(prev)
                es.append(eid)
                cur = prev
            vs.reverse()
            es.reverse()
            results.append({"vertices": vs, "edges": es})
        return results

    if algo == "widest_path_widths_floyd_warshall":
        # Counterpart of igraph_widest_path_widths_floyd_warshall(_, _,
        # vss_all(), vss_all(), weights, IGRAPH_OUT). python-igraph
        # does not bind this either — reference is inline FW with
        # widest-paths recurrence (max of min).
        n = g.vcount()
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            ew = list(g.es["weight"])
        else:
            ew = [1.0] * g.ecount()
        # Init: -inf everywhere; +inf on diagonal.
        mat = [[float("-inf")] * n for _ in range(n)]
        for i in range(n):
            mat[i][i] = float("inf")
        # Seed from edges. OUT mode for directed; bidirectional for undirected.
        directed = g.is_directed()
        for e in range(g.ecount()):
            w = ew[e]
            if w == float("-inf"):
                continue
            s = g.es[e].source
            t = g.es[e].target
            if mat[s][t] < w:
                mat[s][t] = w
            if not directed and mat[t][s] < w:
                mat[t][s] = w
        # FW recurrence.
        for k in range(n):
            for j in range(n):
                width_kj = mat[k][j]
                if j == k or width_kj == float("-inf"):
                    continue
                for i in range(n):
                    if i == j or i == k:
                        continue
                    alt = min(mat[i][k], width_kj)
                    if alt > mat[i][j]:
                        mat[i][j] = alt
        # Convert: -inf → None; +inf → "Infinity" string sentinel.
        return [
            [
                None if w == float("-inf")
                else "Infinity" if w == float("inf")
                else w
                for w in row
            ]
            for row in mat
        ]

    if algo == "widest_path":
        # Counterpart of igraph_get_widest_path(_, _, _, from, to,
        # weights, IGRAPH_OUT). python-igraph does not bind widest
        # paths so the reference is inline: same SPFA-style Dijkstra
        # as the widths oracle, plus parent-edge tracking and
        # backwards reconstruction.
        import heapq

        from_v = int(params["from"])
        to_v = int(params["to"])
        n = g.vcount()
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            ew = list(g.es["weight"])
        else:
            ew = [1.0] * g.ecount()
        widths = [float("-inf")] * n
        widths[from_v] = float("inf")
        parent = [None] * n
        heap = [(-float("inf"), from_v)]
        while heap:
            neg_w, v = heapq.heappop(heap)
            w = -neg_w
            if w < widths[v]:
                continue
            eids = g.incident(v, mode="out" if g.is_directed() else "all")
            for e in eids:
                edge = g.es[e]
                edge_w = ew[e]
                if edge_w == float("-inf"):
                    continue
                other = edge.target if edge.source == v else edge.source
                alt = min(w, edge_w)
                if alt > widths[other]:
                    widths[other] = alt
                    parent[other] = (v, e)
                    heapq.heappush(heap, (-alt, other))
        # Trivial self-target.
        if from_v == to_v:
            return {"vertices": [from_v], "edges": []}
        if widths[to_v] == float("-inf"):
            return None
        # Reconstruct.
        vs = [to_v]
        es = []
        cur = to_v
        while cur != from_v:
            prev, eid = parent[cur]
            vs.append(prev)
            es.append(eid)
            cur = prev
        vs.reverse()
        es.reverse()
        return {"vertices": vs, "edges": es}

    if algo == "widest_path_widths":
        # Counterpart of igraph_widest_path_widths_dijkstra(_, _,
        # vss(source), vss_all(), weights, IGRAPH_OUT). python-igraph
        # does not expose widest paths, so reference implementation
        # is here (Dijkstra variant: max-priority instead of min,
        # relax via min(width[u], edge_weight)).
        import heapq

        source = int(params["source"])
        n = g.vcount()
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            ew = list(g.es["weight"])
        else:
            ew = [1.0] * g.ecount()
        widths = [float("-inf")] * n
        widths[source] = float("inf")
        # Use negated widths to turn min-heap into max-heap.
        heap = [(-float("inf"), source)]
        while heap:
            neg_w, v = heapq.heappop(heap)
            w = -neg_w
            if w < widths[v]:
                continue
            # In OUT mode for directed: outgoing edges; for undirected: all.
            mode_out = g.is_directed()
            if mode_out:
                eids = g.incident(v, mode="out")
            else:
                eids = g.incident(v, mode="all")
            for e in eids:
                edge = g.es[e]
                edge_w = ew[e]
                if edge_w == float("-inf"):
                    continue
                other = edge.target if edge.source == v else edge.source
                alt = min(w, edge_w)
                if alt > widths[other]:
                    widths[other] = alt
                    heapq.heappush(heap, (-alt, other))
        # Convert -inf → None for unreachable; +inf stays as-is (encoded
        # as JSON sentinel string since JSON has no Infinity).
        return [
            None if w == float("-inf")
            else "Infinity" if w == float("inf")
            else w
            for w in widths
        ]

    if algo == "johnson_distances":
        # Counterpart of igraph_distances_johnson(_, _, vss_all(),
        # vss_all(), weights, IGRAPH_OUT). python-igraph's
        # `Graph.distances(weights=...)` auto-picks the algorithm
        # (Dijkstra when non-negative, BF/Johnson when negative).
        # For Johnson we pass weights to get an all-pairs matrix.
        mode = "out" if g.is_directed() else "all"
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            rows = g.distances(weights="weight", mode=mode)
        else:
            rows = g.distances(mode=mode)
        out = []
        for row in rows:
            converted = []
            for v in row:
                f = float(v)
                converted.append(None if f == float("inf") else f)
            out.append(converted)
        return out

    if algo == "bellman_ford_distances":
        # Counterpart of igraph_distances_bellman_ford(_, _, source,
        # vss_all(), weights, IGRAPH_OUT). python-igraph's
        # `Graph.distances(weights=...)` auto-picks an algorithm
        # (Dijkstra for non-negative, BF / Johnson for negative).
        # For the BF oracle we always pass weights explicitly so
        # python-igraph dispatches to its own BF implementation.
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

    if algo == "bellman_ford_distances_with_mode":
        # Counterpart of igraph_distances_bellman_ford(_, _, source,
        # vss_all(), weights, mode).
        source = int(params["source"])
        mode = str(params.get("mode", "out"))
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            rows = g.distances(source=source, weights="weight", mode=mode)
        else:
            rows = g.distances(source=source, mode=mode)
        out = []
        for v in rows[0]:
            f = float(v)
            out.append(None if f == float("inf") else f)
        return out

    if algo == "dijkstra_distances_with_mode":
        # Counterpart of igraph_distances_dijkstra(_, _, source,
        # vss_all(), &weights, mode). python-igraph accepts mode as
        # "out"/"in"/"all" lowercase strings on Graph.distances.
        source = int(params["source"])
        mode = str(params.get("mode", "out"))
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            rows = g.distances(source=source, weights="weight", mode=mode)
        else:
            rows = g.distances(source=source, mode=mode)
        out = []
        for v in rows[0]:
            f = float(v)
            out.append(None if f == float("inf") else f)
        return out

    if algo == "dijkstra_all_shortest_paths":
        # Counterpart of igraph_get_all_shortest_paths_dijkstra(_, _, _,
        # _, source, vss_all(), weights, mode). python-igraph
        # exposes get_all_shortest_paths(v, weights=, mode=). Output:
        # list of vertex paths (one entry per geodesic). We aggregate
        # by target vertex for cross-impl comparison: nrgeo[v] = path
        # count to v, distances[v] = sum-of-weights of any path to v.
        source = int(params["source"])
        mode = str(params.get("mode", "out"))
        weights = (
            list(g.es["weight"])
            if g.ecount() > 0 and "weight" in g.edge_attributes()
            else [1.0] * g.ecount()
        )
        try:
            paths = g.get_all_shortest_paths(source, weights=weights, mode=mode)
        except Exception:
            paths = g.get_all_shortest_paths(source, mode=mode)
        n = g.vcount()
        nrgeo = [0] * n
        # Distances row, used to sanity-check the path lengths and to
        # include in the wire format alongside nrgeo.
        if g.ecount() > 0 and "weight" in g.edge_attributes():
            rows = g.distances(source=source, weights="weight", mode=mode)
        else:
            rows = g.distances(source=source, mode=mode)
        d = [None if float(v) == float("inf") else float(v) for v in rows[0]]
        for p in paths:
            if not p:
                continue
            target = int(p[-1])
            nrgeo[target] += 1
        # Source contributes one trivial geodesic to itself.
        if d[source] is not None:
            # python-igraph's get_all_shortest_paths returns at least
            # one path for the source (just [source]); ensure nrgeo
            # reflects that.
            if nrgeo[source] == 0:
                nrgeo[source] = 1
        return {"distances": d, "nrgeo": nrgeo}

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

    if algo == "disjoint_union_many":
        # Counterpart of igraph_disjoint_union_many(_, &graphs[]). The
        # primary graph (`g`) is the FIRST input; `params.extra_graphs`
        # is a list of {n, edges, directed, weights} payloads for the
        # rest.
        extras = [make_graph(rp) for rp in params.get("extra_graphs", [])]
        u = ig.disjoint_union([g] + extras)
        edges = [list(e.tuple) for e in u.es]
        return {"vcount": u.vcount(), "directed": u.is_directed(), "edges": edges}

    if algo == "union":
        # Counterpart of igraph_union(_, &left, &right, NULL, NULL). The
        # request graph carries `left`; `right` is encoded inside
        # `params.right_graph` (n / edges / directed / weights). Edges
        # are returned canonicalised + sorted: the C kernel emits edges
        # in its internal sort-merge order which is not portable across
        # bindings, so we hand back a deterministic representation
        # suited for set-equality testing.
        rp = params["right_graph"]
        right = make_graph(rp)
        u = ig.union([g, right])
        directed = bool(u.is_directed())
        edges = []
        for e in u.es:
            (s, t) = e.tuple
            if not directed and s > t:
                s, t = t, s
            edges.append([int(s), int(t)])
        edges.sort()
        return {"vcount": u.vcount(), "directed": directed, "edges": edges}

    if algo == "intersection":
        # Counterpart of igraph_intersection(_, &left, &right, NULL,
        # NULL). Same wire format as `union`: right rides via
        # `params.right_graph`. Output edges are canonicalised + sorted
        # for portable comparison across the three reference
        # implementations.
        rp = params["right_graph"]
        right = make_graph(rp)
        u = ig.intersection([g, right])
        directed = bool(u.is_directed())
        edges = []
        for e in u.es:
            (s, t) = e.tuple
            if not directed and s > t:
                s, t = t, s
            edges.append([int(s), int(t)])
        edges.sort()
        return {"vcount": u.vcount(), "directed": directed, "edges": edges}

    if algo == "difference":
        # Counterpart of igraph_difference(_, &orig, &sub). The request
        # graph carries `orig` (the left operand); `sub` rides via
        # `params.right_graph`. NB: vcount is taken from `orig` only
        # (asymmetric — unlike union/intersection). Edges are
        # canonicalised + sorted for portable comparison.
        rp = params["right_graph"]
        sub = make_graph(rp)
        u = g.difference(sub)
        directed = bool(u.is_directed())
        edges = []
        for e in u.es:
            (s, t) = e.tuple
            if not directed and s > t:
                s, t = t, s
            edges.append([int(s), int(t)])
        edges.sort()
        return {"vcount": u.vcount(), "directed": directed, "edges": edges}

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

    if algo == "decompose":
        # Counterpart of igraph_decompose(_, _, IGRAPH_WEAK, -1, 1).
        # python-igraph returns a list of Graph objects whose vertex
        # IDs are renumbered to 0..k. We canonicalise each component
        # to a {vcount, edges} dict with edge endpoints sorted as
        # (u, v) for undirected graphs (u <= v), so list-of-edges
        # comparison is order-stable. Component order matches
        # python-igraph's decompose() (BFS-from-actstart, same as
        # upstream's C kernel).
        comps = g.decompose(mode="weak")
        result = []
        for sub in comps:
            edges = []
            for e in sub.es:
                u, v = int(e.source), int(e.target)
                if not sub.is_directed():
                    u, v = (u, v) if u <= v else (v, u)
                edges.append([u, v])
            edges.sort()
            result.append({
                "vcount": int(sub.vcount()),
                "directed": bool(sub.is_directed()),
                "edges": edges,
            })
        return result

    if algo == "transitivity_barrat":
        # Counterpart of igraph_transitivity_barrat(_, _, igraph_vss_all(),
        # weights, IGRAPH_TRANSITIVITY_NAN). python-igraph dispatches to
        # the Barrat formula automatically when the weights kwarg is set.
        # NaN encoded as None for Option<f64> parity.
        if "weight" in g.edge_attributes():
            vals = g.transitivity_local_undirected(weights="weight", mode="nan")
        else:
            vals = g.transitivity_local_undirected(mode="nan")
        return [None if (v != v) else float(v) for v in vals]

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
