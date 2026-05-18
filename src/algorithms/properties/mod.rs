//! Graph properties — invariants and metrics. Phase 1 entries:
//! ALGO-PR-001 (`girth`), ALGO-PR-002 (triangles + global/local
//! transitivity), ALGO-PR-003 (density + mean distance), ALGO-PR-004
//! (reciprocity), ALGO-PR-005 (avg nearest-neighbour degree),
//! ALGO-PR-006 (degree assortativity).

// `pub(crate)` instead of `pub` so the inner module names (often
// identical to the function they expose, e.g. `is_simple`,
// `pagerank`) don't collide with the function re-exports in the
// rendered rustdoc — see https://github.com/rust-lang/rust/issues/...
pub(crate) mod assortativity;
pub(crate) mod basic;
pub(crate) mod betweenness;
pub(crate) mod betweenness_weighted;
pub(crate) mod closeness;
pub(crate) mod closeness_weighted;
pub(crate) mod edge_betweenness;
pub(crate) mod edge_betweenness_weighted;
pub(crate) mod eigenvector;
pub(crate) mod girth;
pub(crate) mod harmonic;
pub(crate) mod harmonic_weighted;
pub(crate) mod is_simple;
pub(crate) mod knn;
pub(crate) mod multiplicity;
pub(crate) mod pagerank;
pub(crate) mod pagerank_weighted;
pub(crate) mod reciprocity;
pub(crate) mod triangles;

pub use assortativity::assortativity_degree;
pub use basic::{density, mean_distance};
pub use betweenness::betweenness;
pub use betweenness_weighted::betweenness_weighted;
pub use closeness::closeness;
pub use closeness_weighted::closeness_weighted;
pub use edge_betweenness::edge_betweenness;
pub use edge_betweenness_weighted::edge_betweenness_weighted;
pub use eigenvector::eigenvector_centrality;
pub use girth::girth;
pub use harmonic::harmonic_centrality;
pub use harmonic_weighted::harmonic_centrality_weighted;
pub use is_simple::is_simple;
pub use knn::avg_nearest_neighbor_degree;
pub use multiplicity::{has_loop, has_multiple, is_loop, is_multiple};
pub use pagerank::pagerank;
pub use pagerank_weighted::pagerank_weighted;
pub use reciprocity::reciprocity;
pub use triangles::{count_triangles, transitivity_local_undirected, transitivity_undirected};
