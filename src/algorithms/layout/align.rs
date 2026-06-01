//! Layout alignment via nematic tensor (ALGO-LO-016).
//!
//! Counterpart of `igraph_layout_align()` from
//! `references/igraph/src/layout/align.c`.
//!
//! Centers the layout on the coordinate origin and rotates it so the
//! principal axes of the edge-vector distribution align with the
//! coordinate axes. Axes are reordered so the widest extent comes first.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Align a graph layout with the coordinate axes.
///
/// Centers vertex positions on the origin and rotates them so that the
/// layout's principal axes (computed from the nematic tensor of edge
/// directions) coincide with the coordinate axes. After rotation, axes
/// are reordered so the widest extent is in column 0.
///
/// The layout is modified **in place**. Each inner `Vec` represents one
/// vertex's coordinates; all must have the same length (`dim ≥ 1`).
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - `layout.len()` does not equal `graph.vcount()`.
/// - Any coordinate vector differs in length from the first.
/// - `dim == 0` (zero-dimensional coordinates).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_align};
///
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
///
/// // Arbitrary 2D positions
/// let mut layout = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
/// layout_align(&g, &mut layout).unwrap();
///
/// // After alignment, the center of mass is at the origin
/// let cx: f64 = layout.iter().map(|p| p[0]).sum::<f64>() / 3.0;
/// let cy: f64 = layout.iter().map(|p| p[1]).sum::<f64>() / 3.0;
/// assert!((cx).abs() < 1e-10);
/// assert!((cy).abs() < 1e-10);
/// ```
pub fn layout_align(graph: &Graph, layout: &mut [Vec<f64>]) -> IgraphResult<()> {
    let vcount = graph.vcount() as usize;
    let ecount = graph.ecount();

    if layout.len() != vcount {
        return Err(IgraphError::InvalidArgument(
            "layout_align: number of points does not match vertex count".to_string(),
        ));
    }

    if vcount == 0 {
        return Ok(());
    }

    let dim = layout[0].len();
    for (i, pos) in layout.iter().enumerate() {
        if pos.len() != dim {
            return Err(IgraphError::InvalidArgument(format!(
                "layout_align: vertex {i} has {} coordinates, expected {dim}",
                pos.len()
            )));
        }
    }

    if dim == 0 {
        return Err(IgraphError::InvalidArgument(
            "layout_align: coordinates must be at least one-dimensional".to_string(),
        ));
    }

    // Step 1: Center the layout on the origin.
    let mut center = vec![0.0_f64; dim];
    for pos in layout.iter() {
        for (j, &c) in pos.iter().enumerate() {
            center[j] += c;
        }
    }
    let inv_n = 1.0 / vcount as f64;
    for c in &mut center {
        *c *= inv_n;
    }
    for pos in layout.iter_mut() {
        for (j, c) in pos.iter_mut().enumerate() {
            *c -= center[j];
        }
    }

    if dim == 1 {
        return Ok(());
    }

    // Step 2: Compute M_ij = sum_{edges} v_i * v_j  (v = edge vector).
    let mut m_mat = vec![vec![0.0_f64; dim]; dim];
    let mut norm2_sum = 0.0_f64;

    // Correction term for symmetry-breaking (first non-zero contribution).
    let mut correction_saved = false;
    let mut m_correction = vec![vec![0.0_f64; dim]; dim];
    let mut norm2_sum_correction = 0.0_f64;

    for eid in 0..ecount {
        #[allow(clippy::cast_possible_truncation)]
        let (from, to) = graph.edge(eid as u32)?;
        if from == to {
            continue;
        }

        let f = from as usize;
        let t = to as usize;

        for i in 0..dim {
            let vi = layout[f][i] - layout[t][i];
            for j in 0..dim {
                let vj = layout[f][j] - layout[t][j];
                let prod = vi * vj;
                m_mat[i][j] += prod;
                if i == j {
                    norm2_sum += prod;
                }
            }
        }

        if !correction_saved && norm2_sum > 0.0 {
            correction_saved = true;
            norm2_sum_correction = norm2_sum;
            for i in 0..dim {
                for j in 0..dim {
                    m_correction[i][j] = m_mat[i][j];
                }
            }
        }
    }

    // If no edges (or all zero-length), use vertex positions instead.
    if norm2_sum == 0.0 {
        for vid in 0..vcount {
            for i in 0..dim {
                for j in 0..dim {
                    let prod = layout[vid][i] * layout[vid][j];
                    m_mat[i][j] += prod;
                    if i == j {
                        norm2_sum += prod;
                    }
                }
            }

            if !correction_saved && norm2_sum > 0.0 {
                correction_saved = true;
                norm2_sum_correction = norm2_sum;
                for i in 0..dim {
                    for j in 0..dim {
                        m_correction[i][j] = m_mat[i][j];
                    }
                }
            }
        }
    }

    // All vertices at the origin — nothing to align.
    if norm2_sum == 0.0 {
        return Ok(());
    }

    // Step 3: Compute Q = M / norm2_sum - I/dim, find eigenvectors.
    let mut retried = false;
    let rotation;

    loop {
        // Q = M / norm2_sum - I/dim
        let mut q = vec![vec![0.0_f64; dim]; dim];
        let inv_norm2 = 1.0 / norm2_sum;
        let inv_dim = 1.0 / dim as f64;
        for i in 0..dim {
            for j in 0..dim {
                q[i][j] = m_mat[i][j] * inv_norm2;
            }
            q[i][i] -= inv_dim;
        }

        // Eigendecompose Q (symmetric dim×dim matrix).
        let (eigenvalues, eigenvectors) = symmetric_eigen_jacobi(&q);

        // Check if Q is close to zero (rotationally symmetric layout).
        let matrix_norm = eigenvalues.iter().map(|&l| l.abs()).fold(0.0_f64, f64::max);

        if matrix_norm > 1e-3 || retried {
            rotation = eigenvectors;
            break;
        }

        // Remove one contribution to break symmetry, then retry.
        for i in 0..dim {
            for j in 0..dim {
                m_mat[i][j] -= m_correction[i][j];
            }
        }
        norm2_sum -= norm2_sum_correction;

        if norm2_sum <= 0.0 {
            return Ok(());
        }

        retried = true;
    }

    // Step 4: Rotate layout: new_pos = layout * R (R columns = eigenvectors).
    let mut temp = vec![vec![0.0_f64; dim]; vcount];
    for v in 0..vcount {
        for j in 0..dim {
            let mut sum = 0.0_f64;
            for k in 0..dim {
                sum += layout[v][k] * rotation[k][j];
            }
            temp[v][j] = sum;
        }
    }

    // Step 5: Reorder axes by extent (widest first).
    let mut extents: Vec<(f64, usize)> = (0..dim)
        .map(|j| {
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            for v in 0..vcount {
                let c = temp[v][j];
                if c < min {
                    min = c;
                }
                if c > max {
                    max = c;
                }
            }
            (max - min, j)
        })
        .collect();
    extents.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    for v in 0..vcount {
        for (new_j, &(_, old_j)) in extents.iter().enumerate() {
            layout[v][new_j] = temp[v][old_j];
        }
    }

    Ok(())
}

/// Jacobi eigenvalue algorithm for small symmetric matrices.
///
/// Returns `(eigenvalues, eigenvectors)` where `eigenvectors[i][j]` is
/// the `j`-th component of the `i`-th eigenvector. Eigenvalues are
/// sorted by ascending value. Eigenvectors form an orthonormal basis.
fn symmetric_eigen_jacobi(a: &[Vec<f64>]) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = a.len();

    // Work copy — we'll reduce this to diagonal form.
    let mut s: Vec<Vec<f64>> = a.to_vec();

    // Accumulate rotations in V (starts as identity).
    let mut v = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        v[i][i] = 1.0;
    }

    let max_sweeps = 100;

    for _sweep in 0..max_sweeps {
        // Find the largest off-diagonal element.
        let mut max_off = 0.0_f64;
        let mut p = 0;
        let mut q = 1;
        for i in 0..n {
            for j in (i + 1)..n {
                let val = s[i][j].abs();
                if val > max_off {
                    max_off = val;
                    p = i;
                    q = j;
                }
            }
        }

        if max_off < 1e-15 {
            break;
        }

        // Compute rotation angle.
        let (cos, sin) = if (s[p][p] - s[q][q]).abs() < 1e-15 {
            let angle = std::f64::consts::FRAC_PI_4;
            (angle.cos(), angle.sin())
        } else {
            let tau = (s[q][q] - s[p][p]) / (2.0 * s[p][q]);
            let t = if tau >= 0.0 {
                1.0 / (tau + (1.0 + tau * tau).sqrt())
            } else {
                -1.0 / (-tau + (1.0 + tau * tau).sqrt())
            };
            let c = 1.0 / (1.0 + t * t).sqrt();
            (c, t * c)
        };

        // Apply Givens rotation to S.
        let s_pp = s[p][p];
        let s_qq = s[q][q];
        let s_pq = s[p][q];

        s[p][p] = cos * cos * s_pp - 2.0 * sin * cos * s_pq + sin * sin * s_qq;
        s[q][q] = sin * sin * s_pp + 2.0 * sin * cos * s_pq + cos * cos * s_qq;
        s[p][q] = 0.0;
        s[q][p] = 0.0;

        for i in 0..n {
            if i == p || i == q {
                continue;
            }
            let s_ip = s[i][p];
            let s_iq = s[i][q];
            s[i][p] = cos * s_ip - sin * s_iq;
            s[p][i] = s[i][p];
            s[i][q] = sin * s_ip + cos * s_iq;
            s[q][i] = s[i][q];
        }

        // Update eigenvector matrix V.
        for i in 0..n {
            let v_ip = v[i][p];
            let v_iq = v[i][q];
            v[i][p] = cos * v_ip - sin * v_iq;
            v[i][q] = sin * v_ip + cos * v_iq;
        }
    }

    let mut eigenvalues: Vec<f64> = (0..n).map(|i| s[i][i]).collect();

    // Sort eigenvalues ascending and permute eigenvectors accordingly.
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_unstable_by(|&a, &b| {
        eigenvalues[a]
            .partial_cmp(&eigenvalues[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let sorted_eigenvalues: Vec<f64> = indices.iter().map(|&i| eigenvalues[i]).collect();
    eigenvalues = sorted_eigenvalues;

    // eigenvectors[i][j] = column i of V permuted
    let mut sorted_v = vec![vec![0.0_f64; n]; n];
    for row in 0..n {
        for (new_col, &old_col) in indices.iter().enumerate() {
            sorted_v[row][new_col] = v[row][old_col];
        }
    }

    (eigenvalues, sorted_v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    fn center_of_mass(layout: &[Vec<f64>]) -> Vec<f64> {
        let n = layout.len();
        if n == 0 {
            return vec![];
        }
        let dim = layout[0].len();
        let mut c = vec![0.0; dim];
        for pos in layout {
            for (j, &v) in pos.iter().enumerate() {
                c[j] += v;
            }
        }
        for v in &mut c {
            *v /= n as f64;
        }
        c
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let mut layout: Vec<Vec<f64>> = vec![];
        layout_align(&g, &mut layout).unwrap();
        assert!(layout.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let mut layout = vec![vec![5.0, 3.0]];
        layout_align(&g, &mut layout).unwrap();
        // Centered on origin
        assert!(approx_eq(layout[0][0], 0.0));
        assert!(approx_eq(layout[0][1], 0.0));
    }

    #[test]
    fn centering() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let mut layout = vec![vec![10.0, 20.0], vec![12.0, 22.0], vec![14.0, 24.0]];
        layout_align(&g, &mut layout).unwrap();

        let c = center_of_mass(&layout);
        assert!(approx_eq(c[0], 0.0));
        assert!(approx_eq(c[1], 0.0));
    }

    #[test]
    fn preserves_distances() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let mut layout = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];

        // Compute pairwise distances before
        let dist = |a: &[f64], b: &[f64]| -> f64 {
            a.iter()
                .zip(b.iter())
                .map(|(&x, &y)| (x - y) * (x - y))
                .sum::<f64>()
                .sqrt()
        };
        let d01_before = dist(&layout[0], &layout[1]);
        let d12_before = dist(&layout[1], &layout[2]);

        layout_align(&g, &mut layout).unwrap();

        let d01_after = dist(&layout[0], &layout[1]);
        let d12_after = dist(&layout[1], &layout[2]);

        assert!((d01_before - d01_after).abs() < 1e-10);
        assert!((d12_before - d12_after).abs() < 1e-10);
    }

    #[test]
    fn widest_axis_first() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        // Layout with greater extent in Y than X
        let mut layout = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 10.0],
            vec![1.0, 10.0],
        ];
        layout_align(&g, &mut layout).unwrap();

        // After alignment, axis 0 should have the widest extent
        let extent_x = layout
            .iter()
            .map(|p| p[0])
            .fold(f64::NEG_INFINITY, f64::max)
            - layout.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let extent_y = layout
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max)
            - layout.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);

        assert!(
            extent_x >= extent_y - 1e-10,
            "axis 0 extent ({extent_x}) should be >= axis 1 extent ({extent_y})"
        );
    }

    #[test]
    fn one_dimensional() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let mut layout = vec![vec![1.0], vec![3.0], vec![5.0]];
        layout_align(&g, &mut layout).unwrap();
        let c = center_of_mass(&layout);
        assert!(approx_eq(c[0], 0.0));
    }

    #[test]
    fn three_dimensional() {
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        let mut layout = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        layout_align(&g, &mut layout).unwrap();

        let c = center_of_mass(&layout);
        for &ci in &c {
            assert!(approx_eq(ci, 0.0));
        }
    }

    #[test]
    fn no_edges_uses_vertices() {
        let g = Graph::with_vertices(3);
        // Vertices spread along diagonal
        let mut layout = vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![2.0, 2.0]];
        layout_align(&g, &mut layout).unwrap();
        let c = center_of_mass(&layout);
        assert!(approx_eq(c[0], 0.0));
        assert!(approx_eq(c[1], 0.0));
    }

    #[test]
    fn all_at_origin() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        let mut layout = vec![vec![0.0, 0.0], vec![0.0, 0.0], vec![0.0, 0.0]];
        layout_align(&g, &mut layout).unwrap();
        for pos in &layout {
            assert!(approx_eq(pos[0], 0.0));
            assert!(approx_eq(pos[1], 0.0));
        }
    }

    #[test]
    fn self_loops_skipped() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 0).unwrap(); // self-loop
        g.add_edge(0, 1).unwrap();
        let mut layout = vec![vec![0.0, 0.0], vec![1.0, 1.0]];
        layout_align(&g, &mut layout).unwrap();
        let c = center_of_mass(&layout);
        assert!(approx_eq(c[0], 0.0));
        assert!(approx_eq(c[1], 0.0));
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new(3, true).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        let mut layout = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
        layout_align(&g, &mut layout).unwrap();
        let c = center_of_mass(&layout);
        assert!(approx_eq(c[0], 0.0));
        assert!(approx_eq(c[1], 0.0));
    }

    #[test]
    fn mismatched_vertex_count() {
        let g = Graph::with_vertices(3);
        let mut layout = vec![vec![0.0, 0.0], vec![1.0, 1.0]];
        assert!(layout_align(&g, &mut layout).is_err());
    }

    #[test]
    fn inconsistent_dimensions() {
        let g = Graph::with_vertices(2);
        let mut layout = vec![vec![0.0, 0.0], vec![1.0]];
        assert!(layout_align(&g, &mut layout).is_err());
    }

    #[test]
    fn zero_dimensional() {
        let g = Graph::with_vertices(2);
        let mut layout = vec![vec![], vec![]];
        assert!(layout_align(&g, &mut layout).is_err());
    }

    #[test]
    fn jacobi_2x2() {
        // [2  1]
        // [1  3]
        // eigenvalues: (5 ± sqrt(5)) / 2 ≈ 1.382, 3.618
        let m = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let (vals, vecs) = symmetric_eigen_jacobi(&m);

        let expected_0 = (5.0 - 5.0_f64.sqrt()) / 2.0;
        let expected_1 = (5.0 + 5.0_f64.sqrt()) / 2.0;
        assert!((vals[0] - expected_0).abs() < 1e-10);
        assert!((vals[1] - expected_1).abs() < 1e-10);

        // Eigenvectors should be orthonormal
        let dot: f64 = vecs[0][0] * vecs[0][1] + vecs[1][0] * vecs[1][1];
        assert!(dot.abs() < 1e-10);
    }

    #[test]
    fn jacobi_identity() {
        let m = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let (vals, _) = symmetric_eigen_jacobi(&m);
        assert!((vals[0] - 1.0).abs() < 1e-10);
        assert!((vals[1] - 1.0).abs() < 1e-10);
    }
}
