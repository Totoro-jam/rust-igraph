//! Deterministic graph constructors.
//!
//! Counterpart of igraph's `src/constructors/` family: each module here
//! ports one (or a small group of) deterministic generator(s) — graphs
//! whose shape is fully determined by their integer parameters, with no
//! RNG involvement.

pub mod circulant;
pub mod de_bruijn;
pub mod famous;
pub mod full;
pub mod generalized_petersen;
pub mod hamming;
pub mod hypercube;
pub mod kary_tree;
pub mod kautz;
pub mod lcf;
pub mod linegraph;
pub mod mycielskian;
pub mod prufer;
pub mod regular_tree;
pub mod ring;
pub mod square_lattice;
pub mod star;
pub mod symmetric_tree;
pub mod tree_from_parent_vector;
pub mod wheel;
