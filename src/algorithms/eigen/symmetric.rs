//! Symmetric eigenvalue solver via Lanczos iteration.
//!
//! Computes selected eigenvalues and eigenvectors of a real symmetric
//! matrix defined implicitly by a matrix-vector product closure.
//!
//! ```
//! use rust_igraph::eigen_matrix_symmetric;
//! use rust_igraph::EigenWhich;
//!
//! // diag(3, 2, 1): top-2 eigenvalues are 3.0 and 2.0
//! let result = eigen_matrix_symmetric(
//!     3,
//!     |x, y| { y[0] = 3.0*x[0]; y[1] = 2.0*x[1]; y[2] = x[2]; },
//!     2,
//!     EigenWhich::LargestAlgebraic,
//! ).unwrap();
//!
//! assert_eq!(result.eigenvalues.len(), 2);
//! assert!((result.eigenvalues[0] - 3.0).abs() < 1e-6);
//! assert!((result.eigenvalues[1] - 2.0).abs() < 1e-6);
//! ```

use crate::algorithms::community::lanczos;
use crate::core::error::{IgraphError, IgraphResult};

/// Which eigenvalues to compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EigenWhich {
    /// Largest algebraic value (most positive).
    LargestAlgebraic,
    /// Smallest algebraic value (most negative).
    SmallestAlgebraic,
    /// Largest magnitude (largest |λ|).
    LargestMagnitude,
}

/// Result of an eigenvalue decomposition.
#[derive(Debug, Clone)]
pub struct EigenDecomposition {
    /// Computed eigenvalues, ordered according to the requested criterion.
    pub eigenvalues: Vec<f64>,
    /// Corresponding eigenvectors (one `Vec<f64>` per eigenvalue, each of
    /// length `n`).
    pub eigenvectors: Vec<Vec<f64>>,
}

/// Compute selected eigenpairs of a real symmetric matrix.
///
/// The matrix is defined implicitly: `matvec(x, y)` must compute
/// `y = A * x` where both slices have length `n`.
///
/// # Arguments
///
/// * `n` — matrix dimension
/// * `matvec` — closure computing the matrix-vector product `y = A x`
/// * `nev` — number of eigenvalues to compute (clamped to `n`)
/// * `which` — which part of the spectrum to target
///
/// # Errors
///
/// Returns [`IgraphError::InvalidArgument`] if `n == 0`.
pub fn eigen_matrix_symmetric<F>(
    n: usize,
    matvec: F,
    nev: usize,
    which: EigenWhich,
) -> IgraphResult<EigenDecomposition>
where
    F: Fn(&[f64], &mut [f64]),
{
    if n == 0 {
        return Err(IgraphError::InvalidArgument(
            "eigen_matrix_symmetric: matrix dimension must be > 0".into(),
        ));
    }

    let nev = nev.min(n);
    if nev == 0 {
        return Ok(EigenDecomposition {
            eigenvalues: Vec::new(),
            eigenvectors: Vec::new(),
        });
    }

    let internal_which = match which {
        EigenWhich::LargestAlgebraic => lanczos::EigenWhich::LargestAlgebraic,
        EigenWhich::SmallestAlgebraic => lanczos::EigenWhich::SmallestAlgebraic,
        EigenWhich::LargestMagnitude => lanczos::EigenWhich::LargestMagnitude,
    };

    let max_iter = n.saturating_mul(200).max(2000);
    let result = lanczos::lanczos_top_k(n, &matvec, nev, internal_which, max_iter);

    Ok(EigenDecomposition {
        eigenvalues: result.eigenvalues,
        eigenvectors: result.eigenvectors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_eigenvalues() {
        // The identity matrix has eigenvalue 1 with multiplicity n.
        // Lanczos collapses the Krylov subspace for scalar multiples of I,
        // so we request 1 eigenvalue (the only distinct one).
        let result = eigen_matrix_symmetric(
            4,
            |x, y| {
                y[..4].copy_from_slice(&x[..4]);
            },
            1,
            EigenWhich::LargestAlgebraic,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 1);
        assert!(
            (result.eigenvalues[0] - 1.0).abs() < 1e-6,
            "identity eigenvalue should be 1.0, got {}",
            result.eigenvalues[0]
        );
    }

    #[test]
    fn diagonal_matrix_top2() {
        // diag(4, 3, 2, 1) — top-2 eigenvalues should be 4.0 and 3.0
        let diag = [4.0, 3.0, 2.0, 1.0];
        let result = eigen_matrix_symmetric(
            4,
            |x, y| {
                for i in 0..4 {
                    y[i] = diag[i] * x[i];
                }
            },
            2,
            EigenWhich::LargestAlgebraic,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 2);
        assert!(
            (result.eigenvalues[0] - 4.0).abs() < 1e-6,
            "top eigenvalue should be 4.0, got {}",
            result.eigenvalues[0]
        );
        assert!(
            (result.eigenvalues[1] - 3.0).abs() < 1e-6,
            "second eigenvalue should be 3.0, got {}",
            result.eigenvalues[1]
        );
    }

    #[test]
    fn smallest_algebraic() {
        let diag = [4.0, 3.0, 2.0, 1.0];
        let result = eigen_matrix_symmetric(
            4,
            |x, y| {
                for i in 0..4 {
                    y[i] = diag[i] * x[i];
                }
            },
            2,
            EigenWhich::SmallestAlgebraic,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 2);
        assert!(
            (result.eigenvalues[0] - 1.0).abs() < 1e-6,
            "smallest eigenvalue should be 1.0, got {}",
            result.eigenvalues[0]
        );
    }

    #[test]
    fn nev_zero_returns_empty() {
        let result =
            eigen_matrix_symmetric(3, |_x, _y| {}, 0, EigenWhich::LargestAlgebraic).unwrap();
        assert!(result.eigenvalues.is_empty());
    }

    #[test]
    fn n_zero_returns_error() {
        let result = eigen_matrix_symmetric(0, |_x, _y| {}, 1, EigenWhich::LargestAlgebraic);
        assert!(result.is_err());
    }

    #[test]
    fn eigenvectors_orthogonal() {
        // Symmetric matrix with distinct eigenvalues: eigenvectors should be orthogonal
        let diag = [5.0, 3.0, 1.0];
        let result = eigen_matrix_symmetric(
            3,
            |x, y| {
                for i in 0..3 {
                    y[i] = diag[i] * x[i];
                }
            },
            3,
            EigenWhich::LargestAlgebraic,
        )
        .unwrap();

        for i in 0..result.eigenvectors.len() {
            for j in (i + 1)..result.eigenvectors.len() {
                let dot: f64 = result.eigenvectors[i]
                    .iter()
                    .zip(&result.eigenvectors[j])
                    .map(|(a, b)| a * b)
                    .sum();
                assert!(
                    dot.abs() < 1e-6,
                    "eigenvectors {i} and {j} should be orthogonal, dot={dot}"
                );
            }
        }
    }
}
