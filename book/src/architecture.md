# Architecture Decisions

Architecture Decision Records (ADRs) for rust-igraph are maintained on GitHub:

**[View Architecture Decisions on GitHub](https://github.com/Totoro-jam/rust-igraph/blob/main/.codefuse/tracking/ARCHITECTURE.md)**

Key decisions include:

- GPL-2.0-or-later license (matching upstream igraph)
- Zero `unsafe` blocks policy
- Single dependency (`thiserror`) constraint
- Graph data structure design (`Vec<Vec<u32>>` adjacency lists)
- Three-source conformance testing approach
