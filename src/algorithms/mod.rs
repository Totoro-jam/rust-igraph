//! Algorithm implementations for rust-igraph.
//!
//! Phase 0 walking-skeleton scope: only `traversal::bfs` and
//! `io::read_edgelist`. The full algorithm catalog is filled in by AWUs
//! across Phases 1-10 (see `docs/plans/MASTER_PLAN.md`).

pub mod chordality;
pub mod cliques;
pub mod coloring;
pub mod community;
pub mod connectivity;
pub mod constructors;
pub mod flow;
pub mod games;
pub mod io;
pub mod layout;
pub mod matching;
pub mod operators;
pub mod paths;
pub mod properties;
pub mod spanning;
pub mod traversal;
