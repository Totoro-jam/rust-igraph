//! Community-detection algorithms (ALGO-CO-*). Phase 1: `modularity`
//! (Newman-Girvan modularity of a partition). Phase 4: `louvain`
//! multilevel community detection, `leiden` (Traag-Waltman-van Eck 2019).

// `pub(crate)` so the inner module name doesn't double-list with the
// function re-export in rustdoc.
pub(crate) mod leiden;
pub(crate) mod louvain;
pub(crate) mod modularity;

pub use leiden::{
    LEIDEN_DEFAULT_BETA, LEIDEN_DEFAULT_ITERATIONS, LeidenObjective, LeidenOptions, LeidenResult,
    leiden, leiden_weighted, leiden_with_options,
};
pub use louvain::{LouvainResult, louvain, louvain_weighted, louvain_with_options};
pub use modularity::{modularity, modularity_directed, modularity_weighted};
