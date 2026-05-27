//! Graph layout algorithms. Phase 7 entries: ALGO-LO-001 (simple
//! deterministic layouts: circle, star, grid, sphere) and random layouts.
//! ALGO-LO-002: Fruchterman-Reingold force-directed layout.
//! ALGO-LO-003: Kamada-Kawai spring layout.
//! ALGO-LO-004: Reingold-Tilford tree layout.

pub(crate) mod bipartite;
#[allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_range_loop,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::manual_range_contains
)]
pub(crate) mod davidson_harel;
#[allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::unnecessary_cast,
    clippy::needless_for_each,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
pub(crate) mod fruchterman_reingold;
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]
pub(crate) mod gem;
#[allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::too_many_lines,
    clippy::cast_possible_truncation
)]
pub(crate) mod kamada_kawai;
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::needless_range_loop,
    clippy::unnecessary_wraps
)]
pub(crate) mod reingold_tilford;
pub(crate) mod simple;
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::needless_range_loop,
    clippy::too_many_arguments
)]
pub(crate) mod sugiyama;

pub use bipartite::layout_bipartite;
pub use davidson_harel::{DhParams, layout_davidson_harel};
pub use fruchterman_reingold::{
    FrBounds, FrBounds3d, FrGrid, FrParams, FrParams3d, layout_fruchterman_reingold,
    layout_fruchterman_reingold_3d,
};
pub use gem::{GemParams, layout_gem};
pub use kamada_kawai::{
    KkBounds, KkBounds3d, KkParams, KkParams3d, layout_kamada_kawai, layout_kamada_kawai_3d,
};
pub use reingold_tilford::{RtMode, layout_reingold_tilford, layout_reingold_tilford_circular};
pub use simple::{
    layout_circle, layout_grid, layout_grid_3d, layout_random, layout_random_3d, layout_sphere,
    layout_star,
};
pub use sugiyama::{SugiyamaParams, SugiyamaResult, layout_sugiyama};
