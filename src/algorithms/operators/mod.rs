//! Graph operators (ALGO-OP-*). Phase 1: `simplify` (remove loops and/or
//! parallel edges, returning a new [`crate::Graph`]).

// `pub(crate)` so the inner module names don't double-list with the
// function re-exports in rustdoc.
pub(crate) mod bipartite_projection;
pub(crate) mod bipartite_projection_size;
pub(crate) mod complementer;
pub(crate) mod compose;
pub(crate) mod connect_neighborhood;
pub(crate) mod contract_vertices;
pub(crate) mod difference;
pub(crate) mod disjoint_union;
pub(crate) mod even_tarjan;
pub(crate) mod induced_subgraph;
pub(crate) mod induced_subgraph_edges;
pub(crate) mod intersection;
pub(crate) mod is_same_graph;
pub(crate) mod join;
pub(crate) mod permute_vertices;
pub(crate) mod products;
pub(crate) mod residual_graph;
pub(crate) mod reverse;
pub(crate) mod rewire;
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(crate) mod rewire_edges;
pub(crate) mod simplify;
pub(crate) mod subgraph_from_edges;
pub(crate) mod to_directed;
pub(crate) mod to_undirected;
pub(crate) mod union;

pub use bipartite_projection::{BipartiteProjection, bipartite_projection};
pub use bipartite_projection_size::{BipartiteProjectionSize, bipartite_projection_size};
pub use complementer::complementer;
pub use compose::compose;
pub use connect_neighborhood::{connect_neighborhood, graph_power};
pub use contract_vertices::contract_vertices;
pub use difference::difference;
pub use disjoint_union::{disjoint_union, disjoint_union_many};
pub use even_tarjan::{EvenTarjanResult, even_tarjan_reduction};
pub use induced_subgraph::{InducedSubgraphResult, induced_subgraph};
pub use induced_subgraph_edges::induced_subgraph_edges;
pub use intersection::{intersection, intersection_many};
pub use is_same_graph::is_same_graph;
pub use join::join;
pub use permute_vertices::permute_vertices;
pub use products::{
    GraphProductType, cartesian_product, graph_product, lexicographic_product, modular_product,
    rooted_product, strong_product, tensor_product,
};
pub use residual_graph::{ResidualGraphResult, residual_graph, reverse_residual_graph};
pub use reverse::{reverse, reverse_edges};
pub use rewire::rewire;
pub use rewire_edges::{RewireDirectedMode, rewire_directed_edges, rewire_edges};
pub use simplify::simplify;
pub use subgraph_from_edges::{SubgraphFromEdgesResult, subgraph_from_edges};
pub use to_directed::{ToDirectedMode, to_directed};
pub use to_undirected::{ToUndirectedMode, to_undirected};
pub use union::{union, union_many};
