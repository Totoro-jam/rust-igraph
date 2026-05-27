//! Graph operators (ALGO-OP-*). Phase 1: `simplify` (remove loops and/or
//! parallel edges, returning a new [`crate::Graph`]).

// `pub(crate)` so the inner module names don't double-list with the
// function re-exports in rustdoc.
pub(crate) mod complementer;
pub(crate) mod compose;
pub(crate) mod connect_neighborhood;
pub(crate) mod contract_vertices;
pub(crate) mod difference;
pub(crate) mod disjoint_union;
pub(crate) mod induced_subgraph;
pub(crate) mod intersection;
pub(crate) mod is_same_graph;
pub(crate) mod join;
pub(crate) mod permute_vertices;
pub(crate) mod products;
pub(crate) mod reverse;
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(crate) mod rewire_edges;
pub(crate) mod simplify;
pub(crate) mod union;

pub use complementer::complementer;
pub use compose::compose;
pub use connect_neighborhood::{connect_neighborhood, graph_power};
pub use contract_vertices::contract_vertices;
pub use difference::difference;
pub use disjoint_union::{disjoint_union, disjoint_union_many};
pub use induced_subgraph::{InducedSubgraphResult, induced_subgraph};
pub use intersection::intersection;
pub use is_same_graph::is_same_graph;
pub use join::join;
pub use permute_vertices::permute_vertices;
pub use products::{cartesian_product, lexicographic_product, strong_product, tensor_product};
pub use reverse::reverse;
pub use rewire_edges::rewire_edges;
pub use simplify::simplify;
pub use union::union;
