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
pub mod cycles;
pub mod dominating_set;
pub mod edge_cover;
pub mod embedding;
pub mod epidemics;
pub mod feedback_arc_set;
pub mod feedback_vertex_set;
pub mod flow;
pub mod fundamental_cycles;
pub mod games;
pub mod graphlets;
pub mod independent_set;
pub mod io;
pub mod isomorphism;
pub mod layout;
pub mod matching;
pub mod matching_lsap;
pub mod minimum_cycle_basis;
pub mod motifs;
pub mod nongraph;
pub mod operators;
pub mod paths;
pub mod properties;
pub mod simple_cycles;
pub mod spanning;
pub mod spatial;
pub mod traversal;
pub mod vertex_cover;
