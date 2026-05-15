# Security policy

## Reporting a vulnerability

**Please do NOT open a public GitHub issue for security problems.**

Use one of the following private channels:

1. **Preferred — GitHub Security Advisory** (private to maintainers):
   <https://github.com/Totoro-jam/rust-igraph/security/advisories/new>
2. Email: `moqiuchen66@gmail.com` with subject `[rust-igraph security]`.

When reporting, include if you can:
- A minimal reproducer (graph data + the exact API call sequence).
- The version of `rust-igraph`.
- Your assessment of impact.

## Status during pre-1.0 alpha

The crate is in `0.0.x-alpha`; only `Graph`, `read_edgelist`, and `bfs`
ship today. Phase 0 is **not hardened against malicious graph input** —
it assumes inputs have already been validated at the application layer.

Reports about safe-API memory unsafety, attacker-chosen output for
attacker-chosen input, or supply-chain compromise via dependencies are
treated as security issues even now. Reports about OOM on large inputs,
performance, or numerical correctness are bugs (welcome — but not
security).

A formal supported-versions table arrives at 1.0.
