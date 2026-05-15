# TEMPLATE: New algorithm branch for `scripts/oracle.py`.
#
# Copy the function body and the `elif` line into `scripts/oracle.py`'s
# `run()` dispatch, replacing every {{...}} placeholder. The /oracle-add
# skill walks through this.
#
# Wire-format contract: the branch receives the `igraph.Graph` already
# constructed by `make_graph()` plus the test's `params` dict. Return a
# JSON-serializable value matching what the Rust function produces.
#
# Placeholders:
#   {{ALGO_SLUG}}    e.g. "betweenness"
#   {{IG_METHOD}}    e.g. "betweenness"          (python-igraph Graph method)
#   {{ALGO_ID}}      e.g. ALGO-CT-002

# elif algo == "{{ALGO_SLUG}}":
#     # Counterpart of igraph_{{ALGO_SLUG}}().
#     # TODO({{ALGO_ID}}): pull the params the Rust port expects.
#     # Example: vertices = params.get("vertices")  # None == VertexSelector::All
#     #          weights  = params.get("weights")
#     result = g.{{IG_METHOD}}(/* TODO({{ALGO_ID}}): pass params */)
#     # python-igraph often returns numpy arrays or igraph types — coerce to
#     # a plain JSON-serializable list / dict before returning.
#     return list(result)
