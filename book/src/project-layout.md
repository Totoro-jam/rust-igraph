# Project layout

```
rust-igraph/
├── Cargo.toml                         workspace root
├── CLAUDE.md                          project-level rules for AI agents
├── CONTRIBUTING.md                    setup + AWU workflow
├── LICENSE                            GPL-2.0-or-later
├── README.md                          high-level overview
├── deny.toml                          cargo-deny: license + security policy
│
├── crates/
│   ├── igraph-core/                   Graph, Vector, Matrix, error types
│   ├── igraph-algorithms/             every algorithm implementation
│   └── igraph/                        high-level facade + integration tests
│
├── tests/
│   └── conformance/{c,py,r}/<algo>/   3-source fixtures (gitignored: no, tracked)
│
├── benches/                           criterion baselines
├── examples/                          runnable demos
├── fixtures/                          standard graphs (karate, dolphins, ...)
├── scripts/
│   ├── oracle.py                      live python-igraph oracle bridge
│   ├── bench_compare.py               criterion vs python-igraph diff
│   └── test_extract/                  conformance extractors (from_c/py/r)
├── templates/                         Step-3 skeletons for the AWU SOP
│
├── book/                              this mdBook site
├── docs/plans/                        planning + design docs
│
├── .codefuse/tracking/                trackers committed alongside code
│   ├── ALGORITHMS.md                    AWU status (single source of truth)
│   ├── ARCHITECTURE.md                  ADR index
│   ├── CONFORMANCE.md                   3-source coverage matrix
│   ├── AI_PROMPTS.md                    cookbook of working prompts
│   ├── RESUME.md                        session-by-session notes
│   └── perf/<ALGO-XXX>.json             criterion baselines
│
├── .claude/                           AI infrastructure (committed)
│   ├── agents/                          7 sub-agents
│   ├── skills/                          9 /awu-* skills + helpers
│   └── hooks/                           git guardrails + auto-fmt
├── .githooks/                         repo-hosted git hooks (Co-Authored-By)
│
├── references/                        gitignored: igraph C / py / R clones
└── .github/workflows/                 CI + Pages deployment
```
