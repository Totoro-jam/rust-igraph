//! Non-graph utility algorithms (random sampling, etc.).

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::manual_range_contains
)]
pub(crate) mod random_sample;
#[allow(clippy::cast_precision_loss)]
pub(crate) mod sampling;

pub use random_sample::random_sample;
pub use sampling::{sample_dirichlet, sample_sphere_surface, sample_sphere_volume};
