//! Graph operators (ALGO-OP-*). Phase 1: `simplify` (remove loops and/or
//! parallel edges, returning a new [`crate::Graph`]).

pub mod simplify;

pub use simplify::simplify;
