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
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::doc_markdown
)]
pub(crate) mod drl;
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
    clippy::similar_names,
    clippy::too_many_lines
)]
pub(crate) mod gem;
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::items_after_statements,
    clippy::explicit_iter_loop,
    clippy::float_cmp
)]
pub(crate) mod graphopt;
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
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines
)]
pub(crate) mod lgl;
#[allow(
    clippy::cast_precision_loss,
    clippy::needless_range_loop,
    clippy::unnecessary_wraps,
    clippy::float_cmp
)]
pub(crate) mod mds;
#[allow(
    unknown_lints,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::manual_midpoint,
    clippy::too_many_arguments
)]
pub(crate) mod merge_dla;
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
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::float_cmp,
    clippy::needless_range_loop,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::many_single_char_names,
    clippy::doc_markdown,
    clippy::used_underscore_binding
)]
pub(crate) mod umap;

pub use bipartite::layout_bipartite;
pub use davidson_harel::{DhParams, layout_davidson_harel};
pub use drl::{DrlOptions, DrlTemplate, layout_drl};
pub use fruchterman_reingold::{
    FrBounds, FrBounds3d, FrGrid, FrParams, FrParams3d, layout_fruchterman_reingold,
    layout_fruchterman_reingold_3d,
};
pub use gem::{GemParams, layout_gem};
pub use graphopt::{GraphoptParams, layout_graphopt};
pub use kamada_kawai::{
    KkBounds, KkBounds3d, KkParams, KkParams3d, layout_kamada_kawai, layout_kamada_kawai_3d,
};
pub use lgl::{LglParams, layout_lgl};
pub use mds::layout_mds;
pub use merge_dla::layout_merge_dla;
pub use reingold_tilford::{RtMode, layout_reingold_tilford, layout_reingold_tilford_circular};
pub use simple::{
    layout_circle, layout_grid, layout_grid_3d, layout_random, layout_random_3d, layout_sphere,
    layout_star,
};
pub use sugiyama::{SugiyamaParams, SugiyamaResult, layout_sugiyama};
pub use umap::{UmapParams, layout_umap, layout_umap_3d, umap_compute_weights};
