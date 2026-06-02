//! Eigenvalue and eigenvector solvers for matrices and graphs.
//!
//! Provides three levels of access:
//!
//! - [`eigen_matrix_symmetric`] — eigenvalues of a real symmetric matrix
//!   given as a matrix-vector product closure. Uses Lanczos iteration.
//! - [`eigen_matrix`] — eigenvalues of a general (possibly non-symmetric)
//!   real matrix via Arnoldi iteration.
//! - [`eigen_adjacency`] — eigenvalues of a graph's adjacency matrix
//!   (convenience wrapper).
//!
//! All solvers are pure Rust with zero external dependencies.

pub(crate) mod adjacency;
pub(crate) mod general;
pub(crate) mod symmetric;

pub use adjacency::eigen_adjacency;
pub use general::{GeneralEigenDecomposition, GeneralEigenWhich, eigen_matrix};
pub use symmetric::{EigenDecomposition, EigenWhich, eigen_matrix_symmetric};
