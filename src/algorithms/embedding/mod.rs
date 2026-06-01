//! Spectral-embedding utilities.
//!
//! * [`dim_select()`] — Zhu–Ghodsi profile-likelihood dimensionality
//!   selector.
//! * [`adjacency_spectral_embedding()`] — adjacency spectral embedding
//!   (ALGO-EM-002).

pub mod adjacency_spectral_embedding;
pub mod dim_select;

pub use adjacency_spectral_embedding::{
    AdjacencySpectralEmbeddingResult, SpectralWhich, adjacency_spectral_embedding,
};
pub use dim_select::dim_select;
