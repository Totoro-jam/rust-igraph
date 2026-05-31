//! Spectral-embedding utilities.
//!
//! The first member of this module is [`dim_select()`], the Zhu–Ghodsi
//! profile-likelihood dimensionality selector used to pick the number of
//! significant singular values for spectral embeddings.

pub mod dim_select;

pub use dim_select::dim_select;
