//! Graph operators (ALGO-OP-*). Phase 1: `simplify` (remove loops and/or
//! parallel edges, returning a new [`crate::Graph`]).

// `pub(crate)` so the inner module names don't double-list with the
// function re-exports in rustdoc.
pub(crate) mod complementer;
pub(crate) mod disjoint_union;
pub(crate) mod simplify;

pub use complementer::complementer;
pub use disjoint_union::disjoint_union;
pub use simplify::simplify;
