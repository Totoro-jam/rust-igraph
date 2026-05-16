//! Connectivity algorithms. Phase 1: ALGO-CC-001 (weak connected components),
//! ALGO-CC-002 (strongly connected components).

pub mod components;
pub mod strong;

pub use components::{ConnectedComponents, connected_components};
pub use strong::strongly_connected_components;
