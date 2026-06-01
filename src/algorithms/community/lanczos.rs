//! Symmetric Lanczos eigensolver (private).
//!
//! Computes the algebraically largest eigenpair(s) of a real symmetric
//! matrix defined implicitly by a matrix-vector product closure.
//! Used internally by `community_leading_eigenvector` and spectral
//! embedding functions.

/// Result of a Lanczos eigensolver run.
pub(crate) struct LanczosResult {
    pub eigenvalue: f64,
    pub eigenvector: Vec<f64>,
}

/// Compute the algebraically largest eigenpair of a symmetric matrix.
///
/// `matvec(x, y)`: computes `y = A * x`. Both slices have length `n`.
///
/// `start`: initial vector (length `n`). Modified in-place on restart.
///
/// `max_iter`: maximum total matrix-vector products.
#[allow(clippy::many_single_char_names)]
pub(crate) fn lanczos_largest<F>(
    n: usize,
    matvec: &F,
    start: &mut [f64],
    max_iter: usize,
) -> LanczosResult
where
    F: Fn(&[f64], &mut [f64]),
{
    if n == 0 {
        return LanczosResult {
            eigenvalue: 0.0,
            eigenvector: Vec::new(),
        };
    }
    if n == 1 {
        let mut y = vec![0.0];
        matvec(&[1.0], &mut y);
        return LanczosResult {
            eigenvalue: y[0],
            eigenvector: vec![1.0],
        };
    }

    let tol = 1e-10;
    let m = n.min(20);
    let mut total_matvecs = 0usize;

    normalize(start);

    let mut best_eigenvalue = f64::NEG_INFINITY;
    let mut best_eigenvector = vec![0.0; n];

    loop {
        let mut q: Vec<Vec<f64>> = Vec::with_capacity(m + 1);
        let mut alpha_vec = Vec::with_capacity(m);
        let mut beta_vec = Vec::with_capacity(m);

        q.push(start.to_vec());
        let mut w = vec![0.0; n];

        for j in 0..m {
            if total_matvecs >= max_iter {
                break;
            }

            matvec(&q[j], &mut w);
            total_matvecs += 1;

            let a_j: f64 = dot(&q[j], &w);
            alpha_vec.push(a_j);

            for i in 0..n {
                w[i] -= a_j * q[j][i];
            }
            if j > 0 {
                let b_prev = beta_vec[j - 1];
                for i in 0..n {
                    w[i] -= b_prev * q[j - 1][i];
                }
            }

            // Full reorthogonalization
            for prev in &q {
                let proj = dot(prev, &w);
                for i in 0..n {
                    w[i] -= proj * prev[i];
                }
            }

            let b_j = norm(&w);
            if b_j < 1e-14 {
                break;
            }
            beta_vec.push(b_j);

            let inv_b = 1.0 / b_j;
            let q_next: Vec<f64> = w.iter().map(|&x| x * inv_b).collect();
            q.push(q_next);
        }

        if alpha_vec.is_empty() {
            break;
        }

        let (eval, evec_tri) = tridiag_largest_eigenpair(&alpha_vec, &beta_vec);

        let mut v = vec![0.0; n];
        for j in 0..evec_tri.len().min(q.len()) {
            let s_j = evec_tri[j];
            for i in 0..n {
                v[i] += s_j * q[j][i];
            }
        }
        normalize(&mut v);

        let converged = (eval - best_eigenvalue).abs() < tol * (1.0 + eval.abs());
        best_eigenvalue = eval;
        best_eigenvector.copy_from_slice(&v);

        if converged || total_matvecs >= max_iter {
            return LanczosResult {
                eigenvalue: best_eigenvalue,
                eigenvector: best_eigenvector,
            };
        }

        start.copy_from_slice(&v);
    }

    LanczosResult {
        eigenvalue: best_eigenvalue,
        eigenvector: best_eigenvector,
    }
}

/// Result of a top-k Lanczos eigensolver run.
#[allow(dead_code)]
pub(crate) struct LanczosTopKResult {
    /// Eigenvalues in descending order of magnitude or algebraic value.
    pub eigenvalues: Vec<f64>,
    /// Eigenvectors (one per eigenvalue), each of length `n`.
    pub eigenvectors: Vec<Vec<f64>>,
}

/// Which eigenvalues to compute.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EigenWhich {
    /// Largest algebraic (most positive).
    LargestAlgebraic,
    /// Smallest algebraic (most negative).
    SmallestAlgebraic,
    /// Largest magnitude (largest |λ|).
    LargestMagnitude,
}

/// Compute the top-k eigenpairs of a symmetric matrix via Hotelling
/// deflation: find the dominant eigenpair, deflate `A ← A − λ·v·v^T`,
/// repeat. For `SmallestAlgebraic`, negate the matrix first.
///
/// Returns eigenvalues sorted by the requested criterion and their
/// corresponding eigenvectors.
#[allow(clippy::many_single_char_names, dead_code)]
pub(crate) fn lanczos_top_k<F>(
    n: usize,
    matvec: &F,
    k: usize,
    which: EigenWhich,
    max_iter_per: usize,
) -> LanczosTopKResult
where
    F: Fn(&[f64], &mut [f64]),
{
    if n == 0 || k == 0 {
        return LanczosTopKResult {
            eigenvalues: Vec::new(),
            eigenvectors: Vec::new(),
        };
    }

    let actual_k = k.min(n);

    let mut found_eigenvalues: Vec<f64> = Vec::with_capacity(actual_k);
    let mut found_eigenvectors: Vec<Vec<f64>> = Vec::with_capacity(actual_k);
    let mut rng = crate::core::rng::SplitMix64::new(137);

    for _ in 0..actual_k {
        let deflated_count = found_eigenvalues.len();

        let deflated_matvec = |x: &[f64], y: &mut [f64]| {
            match which {
                EigenWhich::SmallestAlgebraic => {
                    matvec(x, y);
                    for val in y.iter_mut() {
                        *val = -*val;
                    }
                }
                _ => {
                    matvec(x, y);
                }
            }
            // Hotelling deflation: subtract λ_i * (v_i^T x) * v_i
            for i in 0..deflated_count {
                let proj: f64 = found_eigenvectors[i]
                    .iter()
                    .zip(x.iter())
                    .map(|(a, b)| a * b)
                    .sum();
                let lam = match which {
                    EigenWhich::SmallestAlgebraic => -found_eigenvalues[i],
                    _ => found_eigenvalues[i],
                };
                for (yj, vj) in y.iter_mut().zip(found_eigenvectors[i].iter()) {
                    *yj -= lam * proj * vj;
                }
            }
        };

        let mut start = vec![0.0_f64; n];
        for (i, sv) in start.iter_mut().enumerate() {
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            *sv = sign + (rng.gen_unit() - 0.5) * 0.2;
        }
        // Orthogonalize start vector against found eigenvectors
        for ev in &found_eigenvectors {
            let proj: f64 = ev.iter().zip(start.iter()).map(|(a, b)| a * b).sum();
            for (sj, vj) in start.iter_mut().zip(ev.iter()) {
                *sj -= proj * vj;
            }
        }

        let result = lanczos_largest(n, &deflated_matvec, &mut start, max_iter_per);

        let eigenvalue = match which {
            EigenWhich::SmallestAlgebraic => -result.eigenvalue,
            _ => result.eigenvalue,
        };

        found_eigenvalues.push(eigenvalue);
        found_eigenvectors.push(result.eigenvector);
    }

    // For LargestMagnitude, sort by |λ| descending
    if which == EigenWhich::LargestMagnitude {
        let mut indices: Vec<usize> = (0..found_eigenvalues.len()).collect();
        indices.sort_by(|&a, &b| {
            found_eigenvalues[b]
                .abs()
                .partial_cmp(&found_eigenvalues[a].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let sorted_evals: Vec<f64> = indices.iter().map(|&i| found_eigenvalues[i]).collect();
        let sorted_evecs: Vec<Vec<f64>> = indices
            .iter()
            .map(|&i| found_eigenvectors[i].clone())
            .collect();
        return LanczosTopKResult {
            eigenvalues: sorted_evals,
            eigenvectors: sorted_evecs,
        };
    }

    LanczosTopKResult {
        eigenvalues: found_eigenvalues,
        eigenvectors: found_eigenvectors,
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn norm(v: &[f64]) -> f64 {
    dot(v, v).sqrt()
}

fn normalize(v: &mut [f64]) {
    let n = norm(v);
    if n > 0.0 {
        let inv = 1.0 / n;
        for x in v.iter_mut() {
            *x *= inv;
        }
    }
}

fn tridiag_largest_eigenpair(alpha: &[f64], beta: &[f64]) -> (f64, Vec<f64>) {
    let n = alpha.len();
    if n == 0 {
        return (0.0, Vec::new());
    }
    if n == 1 {
        return (alpha[0], vec![1.0]);
    }

    let (eigenvalues, eigenvectors) = tridiag_eig(alpha, beta);

    let mut max_idx = 0;
    let mut max_val = eigenvalues[0];
    for (i, &val) in eigenvalues.iter().enumerate().skip(1) {
        if val > max_val {
            max_val = val;
            max_idx = i;
        }
    }

    (max_val, eigenvectors[max_idx].clone())
}

/// Symmetric tridiagonal QL algorithm with implicit shifts.
///
/// Textbook implementation following Numerical Recipes "tqli".
/// `alpha[0..n]` = diagonal, `beta[0..n-1]` = sub-diagonal.
/// Returns `(eigenvalues, eigenvectors_as_rows)`.
#[allow(clippy::many_single_char_names)]
fn tridiag_eig(alpha: &[f64], beta: &[f64]) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = alpha.len();
    if n == 0 {
        return (Vec::new(), Vec::new());
    }

    let mut d = alpha.to_vec();
    let mut e = vec![0.0; n];
    let copy_len = beta.len().min(n - 1);
    e[..copy_len].copy_from_slice(&beta[..copy_len]);

    // z[i][k] accumulates eigenvector components. Starts as identity.
    let mut z = vec![vec![0.0; n]; n];
    for (i, row) in z.iter_mut().enumerate() {
        row[i] = 1.0;
    }

    for l in 0..n {
        let mut iter = 0u32;
        loop {
            let mut m = l;
            while m < n - 1 {
                let dd = d[m].abs() + d[m + 1].abs();
                // Machine-precision negligibility test (Numerical Recipes tqli)
                #[allow(clippy::float_cmp)]
                if e[m].abs() + dd == dd {
                    break;
                }
                m += 1;
            }
            if m == l {
                break;
            }

            iter += 1;
            if iter > 30 {
                break;
            }

            let mut g = (d[l + 1] - d[l]) / (2.0 * e[l]);
            let mut r = g.hypot(1.0);
            g = d[m] - d[l] + e[l] / (g + r.copysign(g));

            let mut s = 1.0;
            let mut c = 1.0;
            let mut p = 0.0;

            for i in (l..m).rev() {
                let f = s * e[i];
                let b = c * e[i];

                r = f.hypot(g);
                e[i + 1] = r;

                if r.abs() < 1e-30 {
                    d[i + 1] -= p;
                    e[m] = 0.0;
                    break;
                }

                s = f / r;
                c = g / r;
                g = d[i + 1] - p;
                r = (d[i] - g) * s + 2.0 * c * b;
                p = s * r;
                d[i + 1] = g + p;
                g = c * r - b;

                // Eigenvector rotation
                let (z_i, z_ip1) = if i + 1 < n {
                    let (lo, hi) = z.split_at_mut(i + 1);
                    (&mut lo[i], &mut hi[0])
                } else {
                    unreachable!()
                };
                for (zk_ip1, zk_i) in z_ip1.iter_mut().zip(z_i.iter_mut()) {
                    let t = *zk_ip1;
                    *zk_ip1 = s * *zk_i + c * t;
                    *zk_i = c * *zk_i - s * t;
                }
            }

            d[l] -= p;
            e[l] = g;
            e[m] = 0.0;
        }
    }

    (d, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tridiag_2x2() {
        // [[2, 1], [1, 2]] => eigenvalues 1, 3
        let (vals, _vecs) = tridiag_eig(&[2.0, 2.0], &[1.0]);
        let mut sorted = vals.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((sorted[0] - 1.0).abs() < 1e-10, "got {sorted:?}");
        assert!((sorted[1] - 3.0).abs() < 1e-10, "got {sorted:?}");
    }

    #[test]
    fn tridiag_5x5_diagonal() {
        let (vals, _) = tridiag_eig(&[1.0, 3.0, 2.0, 5.0, 4.0], &[0.0, 0.0, 0.0, 0.0]);
        let mut sorted = vals.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for (got, expected) in sorted.iter().zip([1.0, 2.0, 3.0, 4.0, 5.0].iter()) {
            assert!(
                (got - expected).abs() < 1e-10,
                "got {got}, expected {expected}"
            );
        }
    }

    #[test]
    fn identity_matrix() {
        let n = 5;
        let matvec = |x: &[f64], y: &mut [f64]| {
            y.copy_from_slice(x);
        };
        let mut start = vec![1.0; n];
        let result = lanczos_largest(n, &matvec, &mut start, 100);
        assert!(
            (result.eigenvalue - 1.0).abs() < 1e-8,
            "got {}",
            result.eigenvalue
        );
    }

    #[test]
    fn diagonal_matrix() {
        let diag = [1.0, 3.0, 2.0, 5.0, 4.0];
        let n = diag.len();
        let matvec = |x: &[f64], y: &mut [f64]| {
            for (yi, (di, xi)) in y.iter_mut().zip(diag.iter().zip(x.iter())) {
                *yi = di * xi;
            }
        };
        let mut start = vec![1.0; n];
        let result = lanczos_largest(n, &matvec, &mut start, 200);
        assert!(
            (result.eigenvalue - 5.0).abs() < 1e-6,
            "expected 5.0, got {}",
            result.eigenvalue
        );
        let max_idx = result
            .eigenvector
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .unwrap()
            .0;
        assert_eq!(max_idx, 3);
    }

    #[test]
    fn symmetric_2x2() {
        let n = 2;
        let matvec = |x: &[f64], y: &mut [f64]| {
            y[0] = 2.0 * x[0] + x[1];
            y[1] = x[0] + 2.0 * x[1];
        };
        let mut start = vec![1.0, 0.5];
        let result = lanczos_largest(n, &matvec, &mut start, 100);
        assert!(
            (result.eigenvalue - 3.0).abs() < 1e-8,
            "expected 3.0, got {}",
            result.eigenvalue
        );
    }

    #[test]
    fn path_graph_laplacian() {
        // Path graph P4: L = [[1,-1,0,0],[-1,2,-1,0],[0,-1,2,-1],[0,0,-1,1]]
        // Eigenvalues: 0, 2-sqrt(2), 2, 2+sqrt(2) ≈ 0, 0.586, 2, 3.414
        let n = 4;
        #[rustfmt::skip]
        let lap = [
            [1.0, -1.0, 0.0, 0.0],
            [-1.0, 2.0, -1.0, 0.0],
            [0.0, -1.0, 2.0, -1.0],
            [0.0, 0.0, -1.0, 1.0],
        ];
        let matvec = |x: &[f64], y: &mut [f64]| {
            for i in 0..n {
                y[i] = 0.0;
                for j in 0..n {
                    y[i] += lap[i][j] * x[j];
                }
            }
        };
        let mut start = vec![1.0, -1.0, 1.0, -1.0];
        let result = lanczos_largest(n, &matvec, &mut start, 200);
        let expected = 2.0 + std::f64::consts::SQRT_2;
        assert!(
            (result.eigenvalue - expected).abs() < 1e-6,
            "expected {expected}, got {}",
            result.eigenvalue
        );
    }
}
