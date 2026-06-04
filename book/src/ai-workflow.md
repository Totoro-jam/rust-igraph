# AI Workflow

rust-igraph uses AI-assisted development with Claude Code. The workflow configuration is on GitHub:

**[View CLAUDE.md on GitHub](https://github.com/Totoro-jam/rust-igraph/blob/main/CLAUDE.md)**

The AI workflow includes:

- **AWU skills** (`/awu-start`, `/awu-translate`, `/awu-test`, etc.) for systematic algorithm porting
- **Sub-agents** for specialized tasks (recon, translation, testing, benchmarking)
- **Three-source conformance extraction** from igraph C, Python, and R test suites
- **Property-based testing** with proptest for algorithmic invariants

All AI-generated code goes through the same quality gates as human-written code: clippy, fmt, full test suite, and conformance checks.
