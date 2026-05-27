//! File I/O for graphs.

// `pub(crate)` to keep rustdoc focused on the function re-export.
pub(crate) mod dimacs;
pub(crate) mod dot;
pub(crate) mod edgelist;
pub(crate) mod gml;
pub(crate) mod lgl;
pub(crate) mod ncol;
pub(crate) mod pajek;

pub use dimacs::{DimacsGraph, DimacsProblem, read_dimacs, write_dimacs_flow};
pub use dot::write_dot;
pub use edgelist::read_edgelist;
pub use gml::{read_gml, write_gml};
pub use lgl::{LglGraph, read_lgl, write_lgl};
pub use ncol::{NcolGraph, read_ncol, write_ncol};
pub use pajek::{PajekGraph, read_pajek, write_pajek};
