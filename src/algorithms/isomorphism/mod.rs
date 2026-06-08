//! Graph isomorphism algorithms (ALGO-ISO-*).
//!
//! First entry: [`simplify_and_colorize`] (`ALGO-ISO-030`) — turn a graph
//! with self-loops and multi-edges into a vertex/edge colored simple
//! graph, the form consumed by isomorphism backends such as VF2.

pub(crate) mod canonical;
pub(crate) mod lad;
pub(crate) mod queries;
pub(crate) mod simplify_and_colorize;
pub(crate) mod subiso;
pub(crate) mod vf2;
pub(crate) mod wl_hash;

pub use canonical::automorphism_group::automorphism_group;
pub use canonical::canonical_permutation::canonical_permutation;
pub use canonical::count_automorphisms::count_automorphisms;
pub use canonical::isomorphic_bliss::isomorphic_bliss;
pub use lad::{LadSubisomorphism, get_subisomorphisms_lad, subisomorphic_lad};
pub use queries::{isomorphic, subisomorphic};
pub use simplify_and_colorize::{SimplifyAndColorize, simplify_and_colorize};
pub use subiso::{
    Vf2Subisomorphism, count_subisomorphisms_vf2, get_subisomorphisms_vf2, subisomorphic_vf2,
};
pub use vf2::{Vf2Isomorphism, count_isomorphisms_vf2, get_isomorphisms_vf2, isomorphic_vf2};
pub use wl_hash::{WlHashResult, wl_hash, wl_hash_iterations, wl_isomorphic};
