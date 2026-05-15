"""Test conformance extractors for the three official igraph implementations.

Each module produces JSON fixtures consumed by `tests/conformance/`. The wire
format is shared with `scripts/oracle.py`:

    {
      "source": "c" | "py" | "r",
      "origin": "<file>:<test-name>",
      "graph":  {"n": int, "edges": [[u, v], ...], "directed": bool,
                 "weights": [float, ...] | null},
      "algo":   str,
      "params": {<algo-specific>},
      "expected": <algo-specific result>
    }
"""
