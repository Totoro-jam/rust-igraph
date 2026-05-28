//! Deterministic graph constructors.
//!
//! Counterpart of igraph's `src/constructors/` family: each module here
//! ports one (or a small group of) deterministic generator(s) — graphs
//! whose shape is fully determined by their integer parameters, with no
//! RNG involvement.

pub mod adjacency;
pub mod atlas;
pub mod atlas_edges;
pub mod biadjacency;
pub mod circulant;
pub mod create;
pub mod create_bipartite;
pub mod de_bruijn;
pub mod extended_chordal_ring;
pub mod famous;
pub mod full;
pub mod full_bipartite;
pub mod full_citation;
pub mod full_multipartite;
pub mod generalized_petersen;
pub mod hamming;
pub mod hexagonal_lattice;
pub mod hypercube;
pub mod kary_tree;
pub mod kautz;
pub mod lcf;
pub mod linegraph;
pub mod mycielskian;
pub mod prufer;
pub mod realize_bipartite_degree_sequence;
pub mod realize_degree_sequence;
pub mod regular_tree;
pub mod ring;
pub mod square_lattice;
pub mod star;
pub mod symmetric_tree;
pub mod tree_from_parent_vector;
pub mod triangular_lattice;
pub mod turan;
pub mod weighted_adjacency;
pub mod wheel;
