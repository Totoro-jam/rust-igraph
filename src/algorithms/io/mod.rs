//! File I/O for graphs. Phase 0 only ships `edgelist`.

// `pub(crate)` to keep rustdoc focused on the function re-export.
pub(crate) mod edgelist;

pub use edgelist::read_edgelist;
