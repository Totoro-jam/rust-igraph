//! Community-detection algorithms (ALGO-CO-*). Phase 1: `modularity`
//! (Newman-Girvan modularity of a partition). Phase 4: `louvain`
//! multilevel community detection.

// `pub(crate)` so the inner module name doesn't double-list with the
// function re-export in rustdoc.
pub(crate) mod louvain;
pub(crate) mod modularity;

pub use louvain::{LouvainResult, louvain, louvain_weighted, louvain_with_options};
pub use modularity::{modularity, modularity_directed, modularity_weighted};
