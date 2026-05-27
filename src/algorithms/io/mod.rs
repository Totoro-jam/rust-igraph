//! File I/O for graphs. Phase 0 shipped `edgelist`; Phase 2 adds GML.

// `pub(crate)` to keep rustdoc focused on the function re-export.
pub(crate) mod edgelist;
pub(crate) mod gml;

pub use edgelist::read_edgelist;
pub use gml::{read_gml, write_gml};
