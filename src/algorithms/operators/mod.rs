//! Graph operators (ALGO-OP-*). Phase 1: `simplify` (remove loops and/or
//! parallel edges, returning a new [`crate::Graph`]).

pub mod disjoint_union;
pub mod simplify;

pub use disjoint_union::disjoint_union;
pub use simplify::simplify;
