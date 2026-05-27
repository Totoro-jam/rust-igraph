//! File I/O for graphs.

// `pub(crate)` to keep rustdoc focused on the function re-export.
pub(crate) mod dimacs;
pub(crate) mod dl;
pub(crate) mod dot;
pub(crate) mod edgelist;
pub(crate) mod gml;
pub(crate) mod graphdb;
pub(crate) mod graphml;
pub(crate) mod leda;
pub(crate) mod lgl;
pub(crate) mod ncol;
pub(crate) mod pajek;

pub use dimacs::{DimacsGraph, DimacsProblem, read_dimacs, write_dimacs_flow};
pub use dl::{DlGraph, read_dl};
pub use dot::write_dot;
pub use edgelist::{read_edgelist, write_edgelist};
pub use gml::{read_gml, write_gml};
pub use graphdb::read_graphdb;
pub use graphml::write_graphml;
pub use leda::write_leda;
pub use lgl::{LglGraph, read_lgl, write_lgl};
pub use ncol::{NcolGraph, read_ncol, write_ncol};
pub use pajek::{PajekGraph, read_pajek, write_pajek};
