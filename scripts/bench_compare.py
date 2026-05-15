#!/usr/bin/env python3
"""Compare per-AWU criterion baseline against a python-igraph timing.

Reads each `.codefuse/tracking/perf/<ALGO-XXX>.json`, runs the equivalent
python-igraph call on the same fixture, fills in the `py_*_ns` slots, and
flags any AWU where Rust is more than `--max-slowdown` × slower.

Output: rewrites the JSON files in place; prints a one-line summary per AWU.

Usage:
    .venv/bin/python -m scripts.bench_compare
    .venv/bin/python -m scripts.bench_compare --max-slowdown 3.0
    .venv/bin/python -m scripts.bench_compare --algo bfs

Phase 0 demo: only `bfs` is wired to a python-igraph call. Add a row to the
PY_CALLS dict for each AWU that needs comparison.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Any, Callable, Dict, Iterable, List

import igraph as ig

REPO_ROOT = Path(__file__).resolve().parents[1]
PERF_DIR = REPO_ROOT / ".codefuse/tracking/perf"
FIXTURES_DIR = REPO_ROOT / "fixtures"


# === Fixture builders (mirror benches/bench_<algo>.rs) ===========


def load_karate() -> ig.Graph:
    edges_path = FIXTURES_DIR / "karate.edges"
    edges: List[tuple[int, int]] = []
    max_id = 0
    with edges_path.open() as f:
        for raw in f:
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            u, v = (int(x) for x in line.split())
            edges.append((u, v))
            max_id = max(max_id, u, v)
    return ig.Graph(n=max_id + 1, edges=edges, directed=False)


def synthetic(n: int) -> ig.Graph:
    edges: List[tuple[int, int]] = []
    for i in range(n - 1):
        edges.append((i, i + 1))
    for i in range(n - 7):
        edges.append((i, i + 7))
    return ig.Graph(n=n, edges=edges, directed=False)


# === Per-algo python-igraph calls =====================================
#
# Each entry maps an AWU's perf-JSON field name (e.g. "rust_karate_ns") to
# (graph_factory, call). The call is what we time; the graph_factory is set
# up *outside* the timed region.

PyCall = Callable[[ig.Graph], Any]
GraphFactory = Callable[[], ig.Graph]

PY_CALLS: Dict[str, Dict[str, tuple[GraphFactory, PyCall]]] = {
    "bfs": {
        "rust_karate_ns": (load_karate, lambda g: g.bfs(0)),
        "rust_synthetic_n1000_ns": (lambda: synthetic(1000), lambda g: g.bfs(0)),
    },
}


def time_ns(graph_factory: GraphFactory, call: PyCall, repeats: int = 50) -> int:
    g = graph_factory()
    samples: List[int] = []
    for _ in range(repeats):
        t0 = time.perf_counter_ns()
        call(g)
        samples.append(time.perf_counter_ns() - t0)
    samples.sort()
    # Take the median to avoid GC noise.
    return samples[len(samples) // 2]


def update_perf(perf_path: Path, max_slowdown: float) -> str:
    data = json.loads(perf_path.read_text())
    awu_id = data.get("awu", "")
    algo = awu_id.split("-")[1].lower() if "-" in awu_id else None
    if not algo:
        return f"{perf_path.name}: no awu id; skipping"
    branches = PY_CALLS.get(algo)
    if branches is None:
        return f"{perf_path.name}: no python-igraph mapping for algo='{algo}'"

    notes: List[str] = []
    for rust_field, (factory, call) in branches.items():
        py_field = rust_field.replace("rust_", "py_")
        if rust_field not in data:
            continue
        rust_ns = data[rust_field]
        if rust_ns is None:
            continue
        py_ns = time_ns(factory, call)
        data[py_field] = py_ns
        ratio = rust_ns / py_ns if py_ns else float("inf")
        verdict = "OK"
        if ratio > max_slowdown:
            verdict = f"PERF-TODO (rust {ratio:.1f}× py)"
        notes.append(
            f"  {rust_field} = {rust_ns} ns vs py {py_ns} ns -> {verdict}"
        )

    perf_path.write_text(json.dumps(data, indent=2) + "\n")
    return f"{perf_path.name}: updated\n" + "\n".join(notes)


def iter_perf_files(only_algo: str | None) -> Iterable[Path]:
    if not PERF_DIR.exists():
        return []
    files = sorted(PERF_DIR.glob("ALGO-*.json"))
    if only_algo is None:
        return files
    needle = f"-{only_algo.upper()}-"
    return [p for p in files if needle in p.name]


def main(argv: List[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--max-slowdown",
        type=float,
        default=3.0,
        help="Threshold ratio rust/python beyond which we flag PERF-TODO (default 3.0).",
    )
    parser.add_argument(
        "--algo",
        help="Restrict to one algorithm (matches the middle of ALGO-<X>-<NNN>).",
    )
    args = parser.parse_args(argv)

    files = list(iter_perf_files(args.algo))
    if not files:
        print(
            "No perf snapshots found under .codefuse/tracking/perf/. "
            "Have any AWUs reached Step 8 (criterion baseline)?",
            file=sys.stderr,
        )
        return 1

    for path in files:
        print(update_perf(path, args.max_slowdown))
    return 0


if __name__ == "__main__":
    sys.exit(main())
