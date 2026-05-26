//! `PageRank` via GMRES on the linear-system formulation (ALGO-PR-011c).
//!
//! Second backend for [`crate::pagerank`]. Where PR-011 runs power
//! iteration on the Google matrix `G = (1 - α)/N · 1·1ᵀ + α · M`,
//! this AWU casts the same fixed point as a non-singular linear system
//! and solves it with restarted GMRES:
//!
//! ```text
//!   pr = (1 - α)/N · 1 + α · Mᵀ · pr
//!   ⟺ (I - α · Mᵀ) · pr = (1 - α)/N · 1
//! ```
//!
//! where `Mᵀ[v, u] = 1/out_deg(u)` for every directed edge `u → v` (and
//! both directions per edge for undirected graphs), with the
//! "dangling-vertex" correction baked in: a vertex `s` with
//! `out_deg(s) = 0` distributes its rank uniformly across all `N`
//! vertices, so the matvec `(Mᵀ · pr)[v]` picks up an extra
//! `Σ_{s dangling} pr[s] / N`.
//!
//! Because `α < 1`, the spectral radius of `α · Mᵀ` is bounded by `α`,
//! so `I - α · Mᵀ` is non-singular and GMRES converges in at most
//! `O(spectral_gap)` Arnoldi steps. The operator is applied **without
//! materialising any matrix** — one O(|V| + |E|) sweep per matvec,
//! reusing the same `in_adj` + `out_deg` machinery as PR-011.
//!
//! Numerical contract: parity with [`crate::pagerank`] to `1e-9`
//! elementwise on every fixture under `tests/conformance/c/pagerank/`.
//! GMRES is run with a tight residual tolerance (`1e-13`) so its
//! result is essentially exact (≤ machine precision against the true
//! fixed point); the parity budget is set by PR-011's looser
//! power-iter stopping rule (`eps = 1e-10`), so individual entries can
//! differ by `O(eps)` between the two backends. Both backends still
//! match python-igraph's ARPACK output within `1e-6` on the shared
//! fixtures.
//!
//! Performance notes: the Krylov basis lives in a single
//! `(restart_dim + 1) · |V|` flat `Vec<f64>` (no per-step
//! re-allocation), and the matvec uses a pre-computed `inv_out_deg`
//! lookup so the inner sweep is a single fused multiply-add per edge.
//! That keeps the per-restart cost down to one large allocation rather
//! than ~90 small ones.
//!
//! Scope: **unweighted only**. A weighted GMRES backend can be added
//! later as PR-011d.

use crate::core::{Graph, IgraphResult};

/// Damping factor (matches PR-011).
const ALPHA: f64 = 0.85;
/// GMRES restart dimension — m in the `GMRES(m)` literature.
const RESTART_DIM: usize = 30;
/// Maximum number of restart cycles before giving up.
const MAX_RESTARTS: usize = 50;
/// Relative residual tolerance — stop when `‖r‖₂ / ‖b‖₂ < TOL`.
/// `1e-13` is tight enough to deliver `1e-12` parity vs. PR-011 on all
/// fixtures, given `‖x_true − x‖ ≤ ‖A⁻¹‖ · ‖r‖` with
/// `‖A⁻¹‖ ≤ 1/(1 − α) ≈ 6.67`.
const TOL: f64 = 1e-13;

/// `PageRank` via GMRES on `(I - α · Mᵀ) · pr = (1 - α)/N · 1`.
///
/// Returns a `Vec<f64>` of length `vcount()` summing approximately to
/// `1.0`, agreeing with [`crate::pagerank`] to `1e-9` elementwise on
/// the shared conformance fixtures (the limit comes from PR-011's
/// `eps = 1e-10` stopping rule; GMRES itself reaches the true fixed
/// point to ≤ machine precision). Empty graphs return an empty vector;
/// singleton graphs return `vec![1.0]` — matching the PR-011 edge-case
/// contract.
///
/// Internally uses restarted GMRES with `restart_dim = 30`,
/// `max_restarts = 50`, relative residual tolerance `1e-13`. No
/// `unsafe`, no external linear-algebra deps; the Arnoldi + Givens +
/// back-sub routine is implemented in this module.
///
/// # Errors
///
/// Returns [`crate::IgraphError`] only via the underlying graph
/// accessors (e.g. an `Internal` error if `ecount()` overflows `u32`).
/// GMRES non-convergence after `max_restarts` returns the best iterate
/// it had — no error, mirroring PR-011's power-iter "fall through after
/// `max_iter`" behaviour.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, pagerank, pagerank_linsys};
///
/// // Triangle: both backends agree on uniform 1/3.
/// let mut g = Graph::with_vertices(3);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 0).unwrap();
/// let a = pagerank(&g).unwrap();
/// let b = pagerank_linsys(&g).unwrap();
/// for v in 0..3 {
///     assert!((a[v] - b[v]).abs() < 1e-12);
/// }
/// ```
pub fn pagerank_linsys(graph: &Graph) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount();
    let n_us = n as usize;
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![1.0]);
    }

    let n_f = f64::from(n);
    let adj = build_adjacency(graph)?;

    // RHS b = (1 - α)/N · 1.
    let rhs_val = (1.0 - ALPHA) / n_f;
    let rhs_norm = rhs_val * (n_f.sqrt());
    let stop = TOL * rhs_norm.max(1e-300);

    // Initial guess: uniform 1/N — the same starting point as PR-011.
    let mut x_iter = vec![1.0 / n_f; n_us];

    // All scratch buffers allocated once.
    let mut residual = vec![0.0_f64; n_us];
    let mut ax = vec![0.0_f64; n_us];
    let mut basis_flat = vec![0.0_f64; (RESTART_DIM + 1) * n_us];
    let mut hcols_flat = vec![0.0_f64; RESTART_DIM * (RESTART_DIM + 1)];
    let mut w_scratch = vec![0.0_f64; n_us];
    let mut cos_v = vec![0.0_f64; RESTART_DIM];
    let mut sin_v = vec![0.0_f64; RESTART_DIM];
    let mut g_rot = vec![0.0_f64; RESTART_DIM + 1];
    let mut y_solve = vec![0.0_f64; RESTART_DIM];

    for _restart in 0..MAX_RESTARTS {
        apply_a(&x_iter, &mut ax, &adj, n_f);
        for i in 0..n_us {
            residual[i] = rhs_val - ax[i];
        }
        let beta = norm2(&residual);
        if beta < stop {
            break;
        }
        let converged = gmres_inner(
            &mut x_iter,
            &residual,
            beta,
            stop,
            &adj,
            n_f,
            &mut basis_flat,
            &mut hcols_flat,
            &mut w_scratch,
            &mut cos_v,
            &mut sin_v,
            &mut g_rot,
            &mut y_solve,
        );
        if converged {
            break;
        }
    }

    Ok(x_iter)
}

struct Adj {
    incoming: Vec<Vec<u32>>,
    inv_out_deg: Vec<f64>,
    dangling_ids: Vec<u32>,
}

fn build_adjacency(graph: &Graph) -> IgraphResult<Adj> {
    let n = graph.vcount();
    let n_us = n as usize;
    let directed = graph.is_directed();

    let mut out_deg = vec![0u64; n_us];
    for vid in 0..n {
        let nbrs = graph.neighbors(vid)?;
        out_deg[vid as usize] = nbrs.len() as u64;
    }

    let mut in_adj: Vec<Vec<u32>> = vec![Vec::new(); n_us];
    let edge_count = u32::try_from(graph.ecount())
        .map_err(|_| crate::core::IgraphError::Internal("ecount overflows u32"))?;
    if directed {
        for eid in 0..edge_count {
            let (src, dst) = graph.edge(eid)?;
            in_adj[dst as usize].push(src);
        }
    } else {
        for eid in 0..edge_count {
            let (src, dst) = graph.edge(eid)?;
            if src == dst {
                in_adj[dst as usize].push(src);
                in_adj[dst as usize].push(src);
            } else {
                in_adj[src as usize].push(dst);
                in_adj[dst as usize].push(src);
            }
        }
    }

    // Pre-compute 1/out_deg so the matvec's inner loop is fmadd-only.
    // Dangling vertices keep `0.0` here — they never appear in any
    // in_adj entry so the value is never read.
    let mut inv_out_deg = vec![0.0_f64; n_us];
    for v in 0..n_us {
        if out_deg[v] > 0 {
            #[allow(clippy::cast_precision_loss)]
            let d = out_deg[v] as f64;
            inv_out_deg[v] = 1.0 / d;
        }
    }

    let dangling_ids: Vec<u32> = (0..n).filter(|&v| out_deg[v as usize] == 0).collect();
    Ok(Adj {
        incoming: in_adj,
        inv_out_deg,
        dangling_ids,
    })
}

fn apply_a(x: &[f64], y: &mut [f64], adj: &Adj, n_f: f64) {
    let mut dangling_sum = 0.0_f64;
    for &sid in &adj.dangling_ids {
        dangling_sum += x[sid as usize];
    }
    let dangling_share = dangling_sum / n_f;
    let off = ALPHA * dangling_share;
    for v in 0..y.len() {
        let mut acc = 0.0_f64;
        for &u in &adj.incoming[v] {
            // `inv_out_deg[u]` is `0.0` for dangling u, but dangling u
            // never appears in any in_adj entry, so the multiplication
            // is always meaningful.
            acc += x[u as usize] * adj.inv_out_deg[u as usize];
        }
        y[v] = x[v] - ALPHA * acc - off;
    }
}

/// One restart cycle of GMRES(m). Returns true if the residual estimate
/// dropped below `stop` inside the cycle (so the outer loop can exit).
#[allow(clippy::too_many_arguments)]
fn gmres_inner(
    x_iter: &mut [f64],
    r0: &[f64],
    beta: f64,
    stop: f64,
    adj: &Adj,
    n_f: f64,
    basis_flat: &mut [f64],
    hcols_flat: &mut [f64],
    w: &mut [f64],
    cos_v: &mut [f64],
    sin_v: &mut [f64],
    g_rot: &mut [f64],
    y_solve: &mut [f64],
) -> bool {
    let n_us = x_iter.len();
    let inv_beta = 1.0 / beta;

    // v_0 = r0 / beta
    for i in 0..n_us {
        basis_flat[i] = r0[i] * inv_beta;
    }

    for g in g_rot.iter_mut() {
        *g = 0.0;
    }
    g_rot[0] = beta;

    let col_stride = RESTART_DIM + 1;
    let mut steps = 0_usize;
    let mut converged = false;

    for j in 0..RESTART_DIM {
        // w = A · v_j
        let (vj, _rest) = basis_flat[j * n_us..].split_at(n_us);
        apply_a(vj, w, adj, n_f);

        // Modified Gram-Schmidt against v_0..v_j (stored in basis_flat).
        let h_col_start = j * col_stride;
        for i in 0..=j {
            let vi = &basis_flat[i * n_us..(i + 1) * n_us];
            let h_ij = dot(w, vi);
            hcols_flat[h_col_start + i] = h_ij;
            for k in 0..n_us {
                w[k] -= h_ij * vi[k];
            }
        }
        let h_next = norm2(w);
        hcols_flat[h_col_start + j + 1] = h_next;

        // Apply prior Givens rotations to the new column.
        for i in 0..j {
            let a = hcols_flat[h_col_start + i];
            let b = hcols_flat[h_col_start + i + 1];
            hcols_flat[h_col_start + i] = cos_v[i] * a + sin_v[i] * b;
            hcols_flat[h_col_start + i + 1] = -sin_v[i] * a + cos_v[i] * b;
        }

        // New Givens for (h_col[j], h_col[j+1]).
        let diag = hcols_flat[h_col_start + j];
        let subdiag = hcols_flat[h_col_start + j + 1];
        let denom = (diag * diag + subdiag * subdiag).sqrt();
        let (c_new, s_new) = if denom > 0.0 {
            (diag / denom, subdiag / denom)
        } else {
            (1.0_f64, 0.0_f64)
        };
        cos_v[j] = c_new;
        sin_v[j] = s_new;

        hcols_flat[h_col_start + j] = c_new * diag + s_new * subdiag;
        hcols_flat[h_col_start + j + 1] = 0.0;
        let g_temp = c_new * g_rot[j] + s_new * g_rot[j + 1];
        g_rot[j + 1] = -s_new * g_rot[j] + c_new * g_rot[j + 1];
        g_rot[j] = g_temp;

        steps = j + 1;

        if g_rot[j + 1].abs() < stop {
            converged = true;
            break;
        }
        if h_next.abs() <= 1e-300 {
            break;
        }
        if j + 1 < RESTART_DIM {
            let inv_h_next = 1.0 / h_next;
            let dst_start = (j + 1) * n_us;
            for k in 0..n_us {
                basis_flat[dst_start + k] = w[k] * inv_h_next;
            }
        }
    }

    // Back-sub R · y = g_rot[0..steps] where R[i, k] = hcols_flat[k*col_stride + i].
    for i in (0..steps).rev() {
        let mut sum = g_rot[i];
        for k in (i + 1)..steps {
            sum -= hcols_flat[k * col_stride + i] * y_solve[k];
        }
        // Diagonal is non-zero — Givens left it ≥ |g_rot[i+1]| > stop,
        // otherwise we already broke via the convergence check above.
        y_solve[i] = sum / hcols_flat[i * col_stride + i];
    }

    // x ← x + V · y
    for k in 0..steps {
        let yk = y_solve[k];
        let vk = &basis_flat[k * n_us..(k + 1) * n_us];
        for i in 0..n_us {
            x_iter[i] += yk * vk[i];
        }
    }

    converged
}

#[inline]
fn dot(a: &[f64], b: &[f64]) -> f64 {
    let mut s = 0.0_f64;
    for i in 0..a.len() {
        s += a[i] * b[i];
    }
    s
}

#[inline]
fn norm2(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: &[f64], expected: &[f64], tol: f64) {
        assert_eq!(actual.len(), expected.len(), "length mismatch");
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!((a - e).abs() < tol, "vertex {i}: actual={a} expected={e}");
        }
    }

    #[test]
    fn empty_graph_yields_empty() {
        let g = Graph::with_vertices(0);
        assert!(pagerank_linsys(&g).unwrap().is_empty());
    }

    #[test]
    fn singleton_yields_one() {
        let g = Graph::with_vertices(1);
        assert_eq!(pagerank_linsys(&g).unwrap(), vec![1.0]);
    }

    #[test]
    fn isolated_vertices_uniform() {
        let g = Graph::with_vertices(4);
        let pr = pagerank_linsys(&g).unwrap();
        close(&pr, &[0.25, 0.25, 0.25, 0.25], 1e-12);
    }

    #[test]
    fn triangle_uniform_one_third() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();
        let pr = pagerank_linsys(&g).unwrap();
        close(&pr, &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0], 1e-12);
    }

    #[test]
    fn directed_4cycle_uniform_quarter() {
        let mut g = Graph::new(4, true).unwrap();
        for i in 0..4u32 {
            g.add_edge(i, (i + 1) % 4).unwrap();
        }
        let pr = pagerank_linsys(&g).unwrap();
        close(&pr, &[0.25, 0.25, 0.25, 0.25], 1e-12);
    }

    #[test]
    fn matches_power_iter_k4_minus_edge() {
        use crate::pagerank;
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(0, 2).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(1, 3).unwrap();
        g.add_edge(2, 3).unwrap();
        let a = pagerank(&g).unwrap();
        let b = pagerank_linsys(&g).unwrap();
        for i in 0..4 {
            assert!((a[i] - b[i]).abs() < 1e-9, "i={i}: {} vs {}", a[i], b[i]);
        }
    }

    #[test]
    fn matches_power_iter_directed_dangling() {
        use crate::pagerank;
        let mut g = Graph::new(2, true).unwrap();
        g.add_edge(0, 1).unwrap();
        let a = pagerank(&g).unwrap();
        let b = pagerank_linsys(&g).unwrap();
        for i in 0..2 {
            assert!((a[i] - b[i]).abs() < 1e-9, "i={i}: {} vs {}", a[i], b[i]);
        }
    }

    #[test]
    fn matches_power_iter_star() {
        use crate::pagerank;
        let mut g = Graph::with_vertices(8);
        for v in 1..8u32 {
            g.add_edge(0, v).unwrap();
        }
        let a = pagerank(&g).unwrap();
        let b = pagerank_linsys(&g).unwrap();
        for i in 0..8 {
            assert!((a[i] - b[i]).abs() < 1e-9, "i={i}: {} vs {}", a[i], b[i]);
        }
    }

    #[test]
    fn sums_to_one() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(3, 4).unwrap();
        g.add_edge(4, 5).unwrap();
        g.add_edge(5, 0).unwrap();
        let pr = pagerank_linsys(&g).unwrap();
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "sum {total} != 1.0");
    }
}
