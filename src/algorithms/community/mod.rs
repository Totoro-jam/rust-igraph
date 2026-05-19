//! Community-detection algorithms (ALGO-CO-*). Phase 1: `modularity`
//! (Newman-Girvan modularity of a partition).

// `pub(crate)` so the inner module name doesn't double-list with the
// function re-export in rustdoc.
pub(crate) mod modularity;

pub use modularity::{modularity, modularity_directed, modularity_weighted};
