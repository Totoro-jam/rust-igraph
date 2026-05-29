//! Connectivity algorithms. Phase 1: ALGO-CC-001 (weak connected components),
//! ALGO-CC-002 (strongly connected components), ALGO-CC-010 (articulation
//! points), ALGO-CC-013 (`is_biconnected`), ALGO-CC-014 (bridges),
//! ALGO-CC-020 (reachability counts), ALGO-CC-021 (reachability matrix),
//! ALGO-CC-022 (transitive closure).

// `pub(crate)` so the inner module names don't double-list with the
// function re-exports in rustdoc.
pub(crate) mod articulation;
pub(crate) mod biconnected;
pub(crate) mod bridges;
pub(crate) mod components;
pub(crate) mod decompose;
pub(crate) mod is_biconnected;
pub(crate) mod is_connected;
pub(crate) mod percolation;
pub(crate) mod reachability;
pub(crate) mod reachability_matrix;
pub(crate) mod separators;
pub(crate) mod strong;
pub(crate) mod subcomponent;
pub(crate) mod transitive_closure;

pub use articulation::articulation_points;
pub use biconnected::{BiconnectedComponents, biconnected_components};
pub use bridges::bridges;
pub use components::{ConnectedComponents, connected_components};
pub use decompose::decompose;
pub use is_biconnected::is_biconnected;
pub use is_connected::{ConnectednessMode, is_connected};
pub use percolation::{
    EdgelistPercolation, SitePercolation, bond_percolation, edgelist_percolation, site_percolation,
};
pub use reachability::count_reachable;
pub use reachability_matrix::reachability_matrix;
pub use separators::{
    all_minimal_st_separators, is_minimal_separator, is_separator, minimum_size_separators,
};
pub use strong::strongly_connected_components;
pub use subcomponent::{SubcomponentMode, subcomponent};
pub use transitive_closure::transitive_closure;
