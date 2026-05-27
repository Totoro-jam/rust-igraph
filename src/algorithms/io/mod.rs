//! File I/O for graphs. Phase 0 shipped `edgelist`; Phase 2 adds GML, NCOL, LGL, DIMACS.

// `pub(crate)` to keep rustdoc focused on the function re-export.
pub(crate) mod dimacs;
pub(crate) mod edgelist;
pub(crate) mod gml;
pub(crate) mod lgl;
pub(crate) mod ncol;

pub use dimacs::{DimacsGraph, DimacsProblem, read_dimacs, write_dimacs_flow};
pub use edgelist::read_edgelist;
pub use gml::{read_gml, write_gml};
pub use lgl::{LglGraph, read_lgl, write_lgl};
pub use ncol::{NcolGraph, read_ncol, write_ncol};
