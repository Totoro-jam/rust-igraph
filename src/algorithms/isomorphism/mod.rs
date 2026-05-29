//! Graph isomorphism algorithms (ALGO-ISO-*).
//!
//! First entry: [`simplify_and_colorize`] (`ALGO-ISO-030`) — turn a graph
//! with self-loops and multi-edges into a vertex/edge colored simple
//! graph, the form consumed by isomorphism backends such as VF2.

pub(crate) mod simplify_and_colorize;

pub use simplify_and_colorize::{SimplifyAndColorize, simplify_and_colorize};
