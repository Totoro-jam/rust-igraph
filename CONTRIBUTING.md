# Contributing — pre-1.0 alpha

**rust-igraph is in `0.0.x-alpha` and is not accepting external pull
requests yet.**

The project is a single-developer + AI port of [igraph](https://igraph.org)
to pure Rust. The public API surface, internal data structures, error
types, and even the crate name are still moving fast; merging external
work right now would mean breaking it on every Phase-1 milestone.

## What you can do today

- ⭐ **Star the repo** if you'd like to follow along.
- 👀 **Watch releases** — we'll cut a `0.1.0` once Phase 1 (the real
  `igraph_t`-equivalent data structures) lands. That's the first version
  worth depending on.
- 🐛 **Bug reports are welcome** — open an issue with a minimal
  reproducer, but expect them to sit in a queue while alpha churn is
  happening.
- 💬 **Discussion / API requests** — use GitHub Discussions when enabled,
  or open an issue tagged `discussion`. Concrete use-cases inform Phase
  ordering.

## What's not accepted today

- New algorithm PRs (algorithms land via the internal AWU SOP — see
  [DEVELOPMENT.md](DEVELOPMENT.md))
- Refactors to data structures that are still `unimplemented!()`
- API renaming bikeshed (the API will rename itself plenty before 1.0)

## When external contributions open up

Roughly when one of these happens:

1. `0.1.0` ships — Phase 1 data structures stabilise.
2. The first algorithm-translation AWUs are merged via the real workflow,
   so the SOP is documented by example rather than by template.
3. CI is hardened against drift (currently passes, but there are no
   external eyeballs on it yet).

When that time comes, this file gets rewritten with real PR conventions
and good-first-issue labels. For now, see
[DEVELOPMENT.md](DEVELOPMENT.md) if you want to read the maintainer's
own setup notes.

## Reporting security issues

See [SECURITY.md](SECURITY.md). Please do **not** open public issues
for vulnerabilities.
