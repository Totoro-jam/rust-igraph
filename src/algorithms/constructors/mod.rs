//! Deterministic graph constructors.
//!
//! Counterpart of igraph's `src/constructors/` family: each module here
//! ports one (or a small group of) deterministic generator(s) — graphs
//! whose shape is fully determined by their integer parameters, with no
//! RNG involvement.

pub mod ring;
pub mod star;
pub mod wheel;
