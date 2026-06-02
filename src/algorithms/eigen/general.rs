//! General (non-symmetric) eigenvalue solver via Arnoldi iteration.
//!
//! Computes selected eigenvalues and eigenvectors of a general real
//! matrix defined implicitly by a matrix-vector product closure.
//!
//! For non-symmetric matrices, eigenvalues may be complex. This module
//! returns eigenvalues as `(real, imag)` pairs and eigenvectors as
//! pairs of real/imaginary component vectors.
//!
//! ```
//! use rust_igraph::eigen_matrix;
//! use rust_igraph::GeneralEigenWhich;
//!
//! // 2×2 rotation matrix [[0, -1], [1, 0]]: eigenvalues are ±i
//! let result = eigen_matrix(
//!     2,
//!     |x, y| { y[0] = -x[1]; y[1] = x[0]; },
//!     2,
//!     GeneralEigenWhich::LargestMagnitude,
//! ).unwrap();
//!
//! assert_eq!(result.eigenvalues.len(), 2);
//! for &(re, im) in &result.eigenvalues {
//!     let mag = (re * re + im * im).sqrt();
//!     assert!((mag - 1.0).abs() < 0.05);
//! }
//! ```

// Numerical linear algebra routines use index-heavy loops and many local
// variables that are standard in the literature (h, q, k, m, n, etc.).
#![allow(
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::manual_memcpy,
    clippy::manual_swap,
    clippy::manual_midpoint
)]

use crate::core::error::{IgraphError, IgraphResult};

/// Which eigenvalues to compute for a general (non-symmetric) matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneralEigenWhich {
    /// Largest magnitude (largest |λ|).
    LargestMagnitude,
    /// Smallest magnitude (smallest |λ|).
    SmallestMagnitude,
    /// Largest real part.
    LargestReal,
    /// Smallest real part (most negative).
    SmallestReal,
}

/// Result of a general eigenvalue decomposition.
///
/// Eigenvalues may be complex, represented as `(real, imaginary)` pairs.
/// Eigenvectors for real eigenvalues have zero imaginary components; for
/// complex conjugate pairs, only one pair of `(real_part, imag_part)`
/// vectors is stored.
#[derive(Debug, Clone)]
pub struct GeneralEigenDecomposition {
    /// Computed eigenvalues as `(real, imaginary)` pairs.
    pub eigenvalues: Vec<(f64, f64)>,
    /// Corresponding eigenvectors: each entry is `(real_part, imag_part)`
    /// where both vectors have length `n`. For a real eigenvalue, the
    /// imaginary part is all zeros.
    pub eigenvectors: Vec<(Vec<f64>, Vec<f64>)>,
}

/// Compute selected eigenpairs of a general real matrix.
///
/// The matrix is defined implicitly: `matvec(x, y)` must compute
/// `y = A * x` where both slices have length `n`.
///
/// Uses Arnoldi iteration to build an upper Hessenberg projection,
/// then QR iteration on the small Hessenberg matrix to extract
/// eigenvalues.
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
///
/// # Examples
///
/// ```
/// use rust_igraph::eigen_matrix;
/// use rust_igraph::GeneralEigenWhich;
///
/// // diag(3, 2, 1): top eigenvalue by magnitude is 3.0
/// let result = eigen_matrix(
///     3,
///     |x, y| { y[0] = 3.0*x[0]; y[1] = 2.0*x[1]; y[2] = x[2]; },
///     2,
///     GeneralEigenWhich::LargestMagnitude,
/// ).unwrap();
///
/// let (re, im) = result.eigenvalues[0];
/// assert!((re - 3.0).abs() < 0.05);
/// assert!(im.abs() < 0.05);
/// ```
pub fn eigen_matrix<F>(
    n: usize,
    matvec: F,
    nev: usize,
    which: GeneralEigenWhich,
) -> IgraphResult<GeneralEigenDecomposition>
where
    F: Fn(&[f64], &mut [f64]),
{
    if n == 0 {
        return Err(IgraphError::InvalidArgument(
            "eigen_matrix: matrix dimension must be > 0".into(),
        ));
    }

    let nev = nev.min(n);
    if nev == 0 {
        return Ok(GeneralEigenDecomposition {
            eigenvalues: Vec::new(),
            eigenvectors: Vec::new(),
        });
    }

    if n == 1 {
        let mut y = vec![0.0];
        matvec(&[1.0], &mut y);
        return Ok(GeneralEigenDecomposition {
            eigenvalues: vec![(y[0], 0.0)],
            eigenvectors: vec![(vec![1.0], vec![0.0])],
        });
    }

    // Arnoldi subspace dimension: larger than nev to get good convergence
    let m = n.min(nev.saturating_mul(2).max(20).min(n));

    // Run Arnoldi iteration
    let (q_basis, h_matrix) = arnoldi_iteration(n, &matvec, m);
    let actual_m = q_basis.len();
    if actual_m == 0 {
        return Ok(GeneralEigenDecomposition {
            eigenvalues: Vec::new(),
            eigenvectors: Vec::new(),
        });
    }

    // Compute eigenvalues of the upper Hessenberg matrix H via QR
    let (eig_re, eig_im) = hessenberg_qr_eigenvalues(&h_matrix, actual_m);

    // Sort eigenvalues by the requested criterion, pair with indices
    let mut indexed: Vec<(usize, f64, f64)> = eig_re
        .iter()
        .zip(eig_im.iter())
        .enumerate()
        .map(|(i, (&re, &im))| (i, re, im))
        .collect();

    match which {
        GeneralEigenWhich::LargestMagnitude => {
            indexed.sort_by(|a, b| {
                let ma = (a.1 * a.1 + a.2 * a.2).sqrt();
                let mb = (b.1 * b.1 + b.2 * b.2).sqrt();
                mb.partial_cmp(&ma).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        GeneralEigenWhich::SmallestMagnitude => {
            indexed.sort_by(|a, b| {
                let ma = (a.1 * a.1 + a.2 * a.2).sqrt();
                let mb = (b.1 * b.1 + b.2 * b.2).sqrt();
                ma.partial_cmp(&mb).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        GeneralEigenWhich::LargestReal => {
            indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }
        GeneralEigenWhich::SmallestReal => {
            indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    // Take top nev
    let selected: Vec<_> = indexed.into_iter().take(nev).collect();

    // Compute Ritz vectors: for each eigenvalue of H, solve (H - λI)z = 0,
    // then x = Q * z gives the approximate eigenvector
    let mut eigenvalues = Vec::with_capacity(nev);
    let mut eigenvectors = Vec::with_capacity(nev);

    for &(idx, re, im) in &selected {
        eigenvalues.push((re, im));

        if im.abs() < 1e-14 {
            // Real eigenvalue: compute real eigenvector of H
            let z = hessenberg_eigenvector_real(&h_matrix, actual_m, re);
            let mut x = vec![0.0; n];
            for (j, zj) in z.iter().enumerate() {
                if j < q_basis.len() {
                    for i in 0..n {
                        x[i] += zj * q_basis[j][i];
                    }
                }
            }
            normalize_vec(&mut x);
            eigenvectors.push((x, vec![0.0; n]));
        } else {
            // Complex eigenvalue: compute complex eigenvector
            let (z_re, z_im) = hessenberg_eigenvector_complex(&h_matrix, actual_m, re, im);
            let mut x_re = vec![0.0; n];
            let mut x_im = vec![0.0; n];
            for j in 0..z_re.len().min(q_basis.len()) {
                for i in 0..n {
                    x_re[i] += z_re[j] * q_basis[j][i];
                    x_im[i] += z_im[j] * q_basis[j][i];
                }
            }
            let norm = {
                let s: f64 = x_re.iter().zip(&x_im).map(|(r, i)| r * r + i * i).sum();
                s.sqrt()
            };
            if norm > 1e-30 {
                for i in 0..n {
                    x_re[i] /= norm;
                    x_im[i] /= norm;
                }
            }
            eigenvectors.push((x_re, x_im));
        }
        let _ = idx;
    }

    Ok(GeneralEigenDecomposition {
        eigenvalues,
        eigenvectors,
    })
}

// ---------------------------------------------------------------
// Arnoldi iteration
// ---------------------------------------------------------------

fn arnoldi_iteration<F>(n: usize, matvec: &F, m: usize) -> (Vec<Vec<f64>>, Vec<Vec<f64>>)
where
    F: Fn(&[f64], &mut [f64]),
{
    let mut q: Vec<Vec<f64>> = Vec::with_capacity(m + 1);
    // H is (m+1) x m upper Hessenberg stored as Vec of columns
    let mut h: Vec<Vec<f64>> = vec![vec![0.0; m + 1]; m];

    // Start with a random-ish vector
    let mut q0 = vec![0.0; n];
    for i in 0..n {
        // Deterministic pseudo-random start via simple hash
        let seed = (i as u64)
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        q0[i] = ((seed >> 33) as f64) / (1u64 << 31) as f64 - 0.5;
    }
    normalize_vec(&mut q0);
    q.push(q0);

    let mut w = vec![0.0; n];

    for j in 0..m {
        matvec(&q[j], &mut w);

        // Modified Gram-Schmidt orthogonalization
        for i in 0..=j {
            let hij = dot_vecs(&q[i], &w);
            h[j][i] = hij;
            for k in 0..n {
                w[k] -= hij * q[i][k];
            }
        }

        // Re-orthogonalize for numerical stability
        for i in 0..=j {
            let corr = dot_vecs(&q[i], &w);
            h[j][i] += corr;
            for k in 0..n {
                w[k] -= corr * q[i][k];
            }
        }

        let beta = norm_vec(&w);
        if beta < 1e-14 {
            // Invariant subspace found
            return (q, h);
        }

        h[j][j + 1] = beta;
        let inv_beta = 1.0 / beta;
        let q_next: Vec<f64> = w.iter().map(|&v| v * inv_beta).collect();
        q.push(q_next);
    }

    (q, h)
}

// ---------------------------------------------------------------
// QR iteration for upper Hessenberg eigenvalues
// ---------------------------------------------------------------

fn hessenberg_qr_eigenvalues(h: &[Vec<f64>], m: usize) -> (Vec<f64>, Vec<f64>) {
    if m == 0 {
        return (Vec::new(), Vec::new());
    }
    if m == 1 {
        return (vec![h[0][0]], vec![0.0]);
    }

    // Copy H into a dense row-major m×m matrix
    let mut a = vec![0.0; m * m];
    for j in 0..m {
        for i in 0..m {
            a[i * m + j] = h[j][i];
        }
    }

    // Implicit double-shift QR (Francis algorithm)
    let max_iter = m.saturating_mul(100).max(1000);
    let mut eig_re = vec![0.0; m];
    let mut eig_im = vec![0.0; m];

    francis_qr(&mut a, m, &mut eig_re, &mut eig_im, max_iter);

    (eig_re, eig_im)
}

/// Francis double-shift QR iteration on an upper Hessenberg matrix.
fn francis_qr(a: &mut [f64], n: usize, eig_re: &mut [f64], eig_im: &mut [f64], max_iter: usize) {
    let mut p = n;
    let mut iter_count = 0;

    while p > 0 && iter_count < max_iter {
        // Find the lowest active subdiagonal element that is negligible
        let mut q = p;
        while q > 1 {
            let h_qq = a[(q - 1) * n + (q - 1)];
            let h_q1q1 = a[(q - 2) * n + (q - 2)];
            let threshold = 1e-14 * (h_qq.abs() + h_q1q1.abs()).max(1e-30);
            if a[(q - 1) * n + (q - 2)].abs() <= threshold {
                a[(q - 1) * n + (q - 2)] = 0.0;
                break;
            }
            q -= 1;
        }

        if q == p {
            // 1×1 block: real eigenvalue
            eig_re[p - 1] = a[(p - 1) * n + (p - 1)];
            eig_im[p - 1] = 0.0;
            p -= 1;
        } else if q == p - 1 {
            // 2×2 block: extract eigenvalues
            let a11 = a[(p - 2) * n + (p - 2)];
            let a12 = a[(p - 2) * n + (p - 1)];
            let a21 = a[(p - 1) * n + (p - 2)];
            let a22 = a[(p - 1) * n + (p - 1)];
            let (e1r, e1i, e2r, e2i) = eigenvalues_2x2(a11, a12, a21, a22);
            eig_re[p - 2] = e1r;
            eig_im[p - 2] = e1i;
            eig_re[p - 1] = e2r;
            eig_im[p - 1] = e2i;
            p -= 2;
        } else {
            // Perform one implicit double-shift QR step
            implicit_double_shift_step(a, n, q.saturating_sub(1), p);
            iter_count += 1;
        }
    }

    // If we didn't converge, extract whatever diagonal we have
    if p > 0 && iter_count >= max_iter {
        for i in 0..p {
            eig_re[i] = a[i * n + i];
            eig_im[i] = 0.0;
        }
    }
}

fn implicit_double_shift_step(a: &mut [f64], n: usize, lo: usize, hi: usize) {
    if hi <= lo + 1 {
        return;
    }

    // Wilkinson shift: eigenvalues of the trailing 2×2 block
    let s = a[(hi - 2) * n + (hi - 2)] + a[(hi - 1) * n + (hi - 1)];
    let t = a[(hi - 2) * n + (hi - 2)] * a[(hi - 1) * n + (hi - 1)]
        - a[(hi - 2) * n + (hi - 1)] * a[(hi - 1) * n + (hi - 2)];

    // First column of (A - σ₁I)(A - σ₂I) = A² - sA + tI
    let mut x = a[lo * n + lo] * a[lo * n + lo] + a[lo * n + (lo + 1)] * a[(lo + 1) * n + lo]
        - s * a[lo * n + lo]
        + t;
    let mut y = a[(lo + 1) * n + lo] * (a[lo * n + lo] + a[(lo + 1) * n + (lo + 1)] - s);
    let mut z = if lo + 2 < hi {
        a[(lo + 1) * n + lo] * a[(lo + 2) * n + (lo + 1)]
    } else {
        0.0
    };

    for k in lo..hi.saturating_sub(1) {
        // Householder reflector to zero out y, z
        let (v0, v1, v2, tau) = householder3(x, y, z);

        let r = k.saturating_sub(1).max(lo);
        let end = hi.min(k + 4);

        // Apply from left: rows k, k+1, k+2
        for j in r..n.min(hi) {
            let sum = v0 * a[k * n + j]
                + v1 * a[(k + 1).min(hi - 1) * n + j]
                + if k + 2 < hi {
                    v2 * a[(k + 2) * n + j]
                } else {
                    0.0
                };
            a[k * n + j] -= tau * v0 * sum;
            a[(k + 1).min(hi - 1) * n + j] -= tau * v1 * sum;
            if k + 2 < hi {
                a[(k + 2) * n + j] -= tau * v2 * sum;
            }
        }

        // Apply from right: columns k, k+1, k+2
        for i in 0..end.min(n) {
            let sum = v0 * a[i * n + k]
                + v1 * a[i * n + (k + 1).min(hi - 1)]
                + if k + 2 < hi {
                    v2 * a[i * n + k + 2]
                } else {
                    0.0
                };
            a[i * n + k] -= tau * v0 * sum;
            a[i * n + (k + 1).min(hi - 1)] -= tau * v1 * sum;
            if k + 2 < hi {
                a[i * n + k + 2] -= tau * v2 * sum;
            }
        }

        // Prepare for next iteration
        if k + 2 < hi.saturating_sub(1) {
            x = a[(k + 1) * n + k];
            y = a[(k + 2) * n + k];
            z = if k + 3 < hi { a[(k + 3) * n + k] } else { 0.0 };
        } else {
            x = a[(k + 1).min(hi - 1) * n + k];
            y = 0.0;
            z = 0.0;
        }
    }
}

fn householder3(x: f64, y: f64, z: f64) -> (f64, f64, f64, f64) {
    let norm = (x * x + y * y + z * z).sqrt();
    if norm < 1e-30 {
        return (1.0, 0.0, 0.0, 0.0);
    }
    let sign_x = if x >= 0.0 { 1.0 } else { -1.0 };
    let v0 = x + sign_x * norm;
    let v1 = y;
    let v2 = z;
    let v_norm_sq = v0 * v0 + v1 * v1 + v2 * v2;
    if v_norm_sq < 1e-30 {
        return (1.0, 0.0, 0.0, 0.0);
    }
    let tau = 2.0 / v_norm_sq;
    (v0, v1, v2, tau)
}

fn eigenvalues_2x2(a: f64, b: f64, c: f64, d: f64) -> (f64, f64, f64, f64) {
    let trace = a + d;
    let det = a * d - b * c;
    let disc = trace * trace - 4.0 * det;
    if disc >= 0.0 {
        let sqrt_disc = disc.sqrt();
        let e1 = (trace + sqrt_disc) / 2.0;
        let e2 = (trace - sqrt_disc) / 2.0;
        (e1, 0.0, e2, 0.0)
    } else {
        let sqrt_disc = (-disc).sqrt();
        let re = trace / 2.0;
        (re, sqrt_disc / 2.0, re, -sqrt_disc / 2.0)
    }
}

// ---------------------------------------------------------------
// Eigenvector computation from Hessenberg Schur form
// ---------------------------------------------------------------

fn hessenberg_eigenvector_real(h: &[Vec<f64>], m: usize, lambda: f64) -> Vec<f64> {
    // Inverse iteration: solve (H - λI)z = 0 approximately
    let mut z = vec![0.0; m];

    // Build (H - λI) in row-major
    let mut mat = vec![0.0; m * m];
    for j in 0..m {
        for i in 0..m {
            mat[i * m + j] = h[j][i];
        }
        mat[j * m + j] -= lambda;
    }

    // Inverse iteration with a random starting vector
    let mut x = vec![0.0; m];
    for i in 0..m {
        let seed = (i as u64)
            .wrapping_mul(2_862_933_555_777_941_757)
            .wrapping_add(3);
        x[i] = ((seed >> 33) as f64) / (1u64 << 31) as f64 - 0.5;
    }

    // Shift slightly to avoid singularity
    let shift = 1e-10 * (1.0 + lambda.abs());
    for i in 0..m {
        mat[i * m + i] += shift;
    }

    // LU factorize (partial pivoting)
    let mut lu = mat.clone();
    let mut piv = vec![0usize; m];
    for i in 0..m {
        piv[i] = i;
    }
    for k in 0..m.saturating_sub(1) {
        let mut max_val = lu[k * m + k].abs();
        let mut max_row = k;
        for i in (k + 1)..m {
            if lu[i * m + k].abs() > max_val {
                max_val = lu[i * m + k].abs();
                max_row = i;
            }
        }
        if max_row != k {
            piv.swap(k, max_row);
            for j in 0..m {
                let tmp = lu[k * m + j];
                lu[k * m + j] = lu[max_row * m + j];
                lu[max_row * m + j] = tmp;
            }
        }
        if lu[k * m + k].abs() < 1e-30 {
            continue;
        }
        for i in (k + 1)..m {
            lu[i * m + k] /= lu[k * m + k];
            for j in (k + 1)..m {
                lu[i * m + j] -= lu[i * m + k] * lu[k * m + j];
            }
        }
    }

    // 3 rounds of inverse iteration
    for _ in 0..3 {
        // Apply permutation
        let mut b = vec![0.0; m];
        for i in 0..m {
            b[i] = x[piv[i]];
        }
        // Forward substitution
        for i in 1..m {
            for j in 0..i {
                b[i] -= lu[i * m + j] * b[j];
            }
        }
        // Back substitution
        for i in (0..m).rev() {
            for j in (i + 1)..m {
                b[i] -= lu[i * m + j] * b[j];
            }
            if lu[i * m + i].abs() > 1e-30 {
                b[i] /= lu[i * m + i];
            }
        }
        x = b;
        normalize_vec(&mut x);
    }

    z.copy_from_slice(&x);
    z
}

fn hessenberg_eigenvector_complex(
    h: &[Vec<f64>],
    m: usize,
    lambda_re: f64,
    lambda_im: f64,
) -> (Vec<f64>, Vec<f64>) {
    // For complex eigenvalues, we solve (H - λI)z = 0 in complex arithmetic
    // using inverse iteration with complex vectors
    let mut z_re = vec![0.0; m];
    let mut z_im = vec![0.0; m];

    // Build (H - λ_re·I) and store -λ_im for imaginary part
    let mut mat_re = vec![0.0; m * m];
    for j in 0..m {
        for i in 0..m {
            mat_re[i * m + j] = h[j][i];
        }
        mat_re[j * m + j] -= lambda_re;
    }

    // Start with a random complex vector
    let mut x_re = vec![0.0; m];
    let mut x_im = vec![0.0; m];
    for i in 0..m {
        let seed1 = (i as u64)
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(7);
        let seed2 = (i as u64)
            .wrapping_mul(2_862_933_555_777_941_757)
            .wrapping_add(13);
        x_re[i] = ((seed1 >> 33) as f64) / (1u64 << 31) as f64 - 0.5;
        x_im[i] = ((seed2 >> 33) as f64) / (1u64 << 31) as f64 - 0.5;
    }

    // Use real Schur form approach: build the 2m×2m real system
    // [ (H-re·I)  im·I ] [x_re]   [0]
    // [ -im·I   (H-re·I)] [x_im] = [0]
    // and solve via inverse iteration on the real system
    let shift = 1e-10 * (1.0 + lambda_re.abs() + lambda_im.abs());
    let n2 = 2 * m;
    let mut big_mat = vec![0.0; n2 * n2];
    for i in 0..m {
        for j in 0..m {
            big_mat[i * n2 + j] = mat_re[i * m + j];
            big_mat[(i + m) * n2 + (j + m)] = mat_re[i * m + j];
        }
        big_mat[i * n2 + (i + m)] = lambda_im;
        big_mat[(i + m) * n2 + i] = -lambda_im;
        big_mat[i * n2 + i] += shift;
        big_mat[(i + m) * n2 + (i + m)] += shift;
    }

    // LU factorize big_mat
    let mut lu = big_mat;
    let mut piv = vec![0usize; n2];
    for i in 0..n2 {
        piv[i] = i;
    }
    for k in 0..n2.saturating_sub(1) {
        let mut max_val = lu[k * n2 + k].abs();
        let mut max_row = k;
        for i in (k + 1)..n2 {
            if lu[i * n2 + k].abs() > max_val {
                max_val = lu[i * n2 + k].abs();
                max_row = i;
            }
        }
        if max_row != k {
            piv.swap(k, max_row);
            for j in 0..n2 {
                let tmp = lu[k * n2 + j];
                lu[k * n2 + j] = lu[max_row * n2 + j];
                lu[max_row * n2 + j] = tmp;
            }
        }
        if lu[k * n2 + k].abs() < 1e-30 {
            continue;
        }
        for i in (k + 1)..n2 {
            lu[i * n2 + k] /= lu[k * n2 + k];
            for j in (k + 1)..n2 {
                lu[i * n2 + j] -= lu[i * n2 + k] * lu[k * n2 + j];
            }
        }
    }

    // Combined start vector
    let mut big_x = vec![0.0; n2];
    for i in 0..m {
        big_x[i] = x_re[i];
        big_x[i + m] = x_im[i];
    }

    // 3 rounds of inverse iteration
    for _ in 0..3 {
        let mut b = vec![0.0; n2];
        for i in 0..n2 {
            b[i] = big_x[piv[i]];
        }
        for i in 1..n2 {
            for j in 0..i {
                b[i] -= lu[i * n2 + j] * b[j];
            }
        }
        for i in (0..n2).rev() {
            for j in (i + 1)..n2 {
                b[i] -= lu[i * n2 + j] * b[j];
            }
            if lu[i * n2 + i].abs() > 1e-30 {
                b[i] /= lu[i * n2 + i];
            }
        }
        let norm: f64 = b.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-30 {
            for v in &mut b {
                *v /= norm;
            }
        }
        big_x = b;
    }

    for i in 0..m {
        z_re[i] = big_x[i];
        z_im[i] = big_x[i + m];
    }

    (z_re, z_im)
}

// ---------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------

fn dot_vecs(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn norm_vec(v: &[f64]) -> f64 {
    dot_vecs(v, v).sqrt()
}

fn normalize_vec(v: &mut [f64]) {
    let n = norm_vec(v);
    if n > 1e-30 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagonal_real_eigenvalues() {
        let result = eigen_matrix(
            3,
            |x, y| {
                y[0] = 3.0 * x[0];
                y[1] = 2.0 * x[1];
                y[2] = x[2];
            },
            2,
            GeneralEigenWhich::LargestMagnitude,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 2);
        let (re0, im0) = result.eigenvalues[0];
        assert!(
            (re0 - 3.0).abs() < 0.05,
            "top eigenvalue should be ~3.0, got {re0}"
        );
        assert!(im0.abs() < 0.05, "should be real, got im={im0}");
    }

    #[test]
    fn rotation_matrix_complex_eigenvalues() {
        // [[0, -1], [1, 0]] has eigenvalues ±i
        let result = eigen_matrix(
            2,
            |x, y| {
                y[0] = -x[1];
                y[1] = x[0];
            },
            2,
            GeneralEigenWhich::LargestMagnitude,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 2);
        for &(re, im) in &result.eigenvalues {
            let mag = (re * re + im * im).sqrt();
            assert!(
                (mag - 1.0).abs() < 1e-4,
                "eigenvalue magnitude should be 1.0, got {mag} (re={re}, im={im})"
            );
            assert!(re.abs() < 1e-4, "real part should be ~0, got {re}");
        }
    }

    #[test]
    fn upper_triangular_eigenvalues() {
        // [[2, 1], [0, 5]]: eigenvalues are 2 and 5
        let result = eigen_matrix(
            2,
            |x, y| {
                y[0] = 2.0 * x[0] + x[1];
                y[1] = 5.0 * x[1];
            },
            2,
            GeneralEigenWhich::LargestMagnitude,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 2);
        let (re0, _) = result.eigenvalues[0];
        let (re1, _) = result.eigenvalues[1];
        let mut vals = [re0, re1];
        vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        assert!(
            (vals[0] - 5.0).abs() < 0.1,
            "largest eigenvalue should be ~5.0, got {}",
            vals[0]
        );
        assert!(
            (vals[1] - 2.0).abs() < 0.1,
            "second eigenvalue should be ~2.0, got {}",
            vals[1]
        );
    }

    #[test]
    fn nev_zero_returns_empty() {
        let result = eigen_matrix(3, |_x, _y| {}, 0, GeneralEigenWhich::LargestMagnitude).unwrap();
        assert!(result.eigenvalues.is_empty());
    }

    #[test]
    fn n_zero_returns_error() {
        let result = eigen_matrix(0, |_x, _y| {}, 1, GeneralEigenWhich::LargestMagnitude);
        assert!(result.is_err());
    }

    #[test]
    fn n_one_returns_single() {
        let result = eigen_matrix(
            1,
            |x, y| {
                y[0] = 7.0 * x[0];
            },
            1,
            GeneralEigenWhich::LargestMagnitude,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 1);
        let (re, im) = result.eigenvalues[0];
        assert!(
            (re - 7.0).abs() < 1e-6,
            "eigenvalue should be 7.0, got {re}"
        );
        assert!(im.abs() < 1e-10);
    }

    #[test]
    fn smallest_real_selection() {
        let result = eigen_matrix(
            3,
            |x, y| {
                y[0] = -5.0 * x[0];
                y[1] = 2.0 * x[1];
                y[2] = 10.0 * x[2];
            },
            1,
            GeneralEigenWhich::SmallestReal,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 1);
        let (re, _) = result.eigenvalues[0];
        assert!(
            (re - (-5.0)).abs() < 0.1,
            "smallest real eigenvalue should be ~-5.0, got {re}"
        );
    }

    #[test]
    fn largest_real_selection() {
        let result = eigen_matrix(
            3,
            |x, y| {
                y[0] = -5.0 * x[0];
                y[1] = 2.0 * x[1];
                y[2] = 10.0 * x[2];
            },
            1,
            GeneralEigenWhich::LargestReal,
        )
        .unwrap();

        assert_eq!(result.eigenvalues.len(), 1);
        let (re, _) = result.eigenvalues[0];
        assert!(
            (re - 10.0).abs() < 0.1,
            "largest real eigenvalue should be ~10.0, got {re}"
        );
    }
}
