//! Kamada-Kawai spring layout (ALGO-LO-003).
//!
//! Energy-minimization layout based on graph-theoretic distances.
//! Reference: Kamada & Kawai, "An Algorithm for Drawing General
//! Undirected Graphs", Information Processing Letters 31 (1989).

use crate::algorithms::paths::floyd_warshall::floyd_warshall_distances;
use crate::core::{Graph, IgraphError, IgraphResult};

// Threshold for near-zero gradient (skip move) — matches igraph C.
const KK_EPS: f64 = 1e-13;
const KK3D_EPS: f64 = 1e-8;

/// Parameters for the 2D Kamada-Kawai layout.
#[derive(Debug, Clone)]
pub struct KkParams {
    /// Maximum number of iterations. Default: `10 * vcount`.
    pub maxiter: usize,
    /// Stop when the maximum energy-gradient magnitude squared falls
    /// below this threshold. Zero means run for exactly `maxiter` iterations.
    pub epsilon: f64,
    /// Spring constant K. Default: `vcount` as f64.
    pub kkconst: f64,
    /// Optional initial positions (row per vertex: `[x, y]`).
    /// If `None`, a scaled circle layout is used.
    pub seed: Option<Vec<[f64; 2]>>,
}

/// Per-vertex coordinate bounds for the 2D KK layout.
#[derive(Debug, Clone, Default)]
pub struct KkBounds {
    pub minx: Option<Vec<f64>>,
    pub maxx: Option<Vec<f64>>,
    pub miny: Option<Vec<f64>>,
    pub maxy: Option<Vec<f64>>,
}

/// Parameters for the 3D Kamada-Kawai layout.
#[derive(Debug, Clone)]
pub struct KkParams3d {
    /// Maximum number of iterations. Default: `10 * vcount`.
    pub maxiter: usize,
    /// Stop when the maximum energy-gradient magnitude squared falls
    /// below this threshold.
    pub epsilon: f64,
    /// Spring constant K. Default: `vcount` as f64.
    pub kkconst: f64,
    /// Optional initial positions (row per vertex: `[x, y, z]`).
    /// If `None`, a scaled sphere layout is used.
    pub seed: Option<Vec<[f64; 3]>>,
}

/// Per-vertex coordinate bounds for the 3D KK layout.
#[derive(Debug, Clone, Default)]
pub struct KkBounds3d {
    pub minx: Option<Vec<f64>>,
    pub maxx: Option<Vec<f64>>,
    pub miny: Option<Vec<f64>>,
    pub maxy: Option<Vec<f64>>,
    pub minz: Option<Vec<f64>>,
    pub maxz: Option<Vec<f64>>,
}

/// Compute 2D Kamada-Kawai layout.
///
/// Places vertices so that graph-theoretic distance is reflected in
/// Euclidean distance. Works well for small-to-medium graphs (O(V²)
/// memory for distance matrices).
///
/// # Arguments
///
/// * `graph` — input graph (treated as undirected for shortest paths).
/// * `weights` — optional edge weights interpreted as edge *lengths*
///   (higher weight → vertices farther apart). Must be positive.
/// * `params` — algorithm parameters; use [`KkParams::default_for`] for
///   reasonable defaults.
/// * `bounds` — optional per-vertex coordinate bounds.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_kamada_kawai, KkParams};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
///
/// let params = KkParams::default_for(4);
/// let pos = layout_kamada_kawai(&g, None, &params, None).unwrap();
/// assert_eq!(pos.len(), 4);
/// ```
pub fn layout_kamada_kawai(
    graph: &Graph,
    weights: Option<&[f64]>,
    params: &KkParams,
    bounds: Option<&KkBounds>,
) -> IgraphResult<Vec<[f64; 2]>> {
    let n = graph.vcount() as usize;

    if params.kkconst <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "kkconst must be positive".into(),
        ));
    }
    if let Some(ws) = weights {
        if ws.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(
                "weights length must equal edge count".into(),
            ));
        }
        if graph.ecount() > 0 {
            for &w in ws {
                if w <= 0.0 || w.is_nan() {
                    return Err(IgraphError::InvalidArgument(
                        "weights must be positive for Kamada-Kawai layout".into(),
                    ));
                }
            }
        }
    }

    validate_bounds_2d(n, bounds)?;

    // Initialize positions
    let mut pos = match &params.seed {
        Some(seed) => {
            if seed.len() != n {
                return Err(IgraphError::InvalidArgument(
                    "seed length must equal vertex count".into(),
                ));
            }
            seed.clone()
        }
        None => {
            if bounds.is_some() {
                initial_bounded_2d(n, bounds)
            } else {
                initial_circle_2d(n)
            }
        }
    };

    // Clamp initial positions to bounds
    if let Some(b) = bounds {
        clamp_positions_2d(&mut pos, b);
    }

    if n <= 1 {
        return Ok(pos);
    }

    // All-pairs shortest paths
    let mut dij = all_pairs_distances(graph, weights)?;

    // Find max finite distance, replace infinite with max
    let max_dij = find_max_finite(&mut dij, n);
    if max_dij == 0.0 {
        return Ok(pos);
    }

    let l0 = (n as f64).sqrt();
    let big_l = l0 / max_dij;

    // Build kij and lij matrices (flat, row-major)
    let mut kij = vec![0.0_f64; n * n];
    let mut lij = vec![0.0_f64; n * n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let d = dij[i * n + j];
            let d2 = d * d;
            if d2 > 0.0 {
                kij[i * n + j] = params.kkconst / d2;
                lij[i * n + j] = big_l * d;
            }
        }
    }

    // Initialize gradient vectors D1 (∂E/∂x_m), D2 (∂E/∂y_m)
    let mut d1 = vec![0.0_f64; n];
    let mut d2 = vec![0.0_f64; n];
    for m in 0..n {
        let (gx, gy) = gradient_2d(m, &pos, &kij, &lij, n);
        d1[m] = gx;
        d2[m] = gy;
    }

    // Main iteration
    for _iter in 0..params.maxiter {
        // Find vertex with maximum delta (energy gradient magnitude²)
        let (m, max_delta) = select_max_delta_2d(&d1, &d2, n);
        if max_delta < params.epsilon {
            break;
        }

        let old_x = pos[m][0];
        let old_y = pos[m][1];

        // Build the 2×2 system coefficients (Hessian of energy w.r.t. m)
        let mut axx = 0.0_f64;
        let mut axy = 0.0_f64;
        let mut ayy = 0.0_f64;
        for i in 0..n {
            if i == m {
                continue;
            }
            let dx = old_x - pos[i][0];
            let dy = old_y - pos[i][1];
            let dist = (dx * dx + dy * dy).sqrt();
            let den = dist * (dx * dx + dy * dy);
            let k_mi = kij[m * n + i];
            let l_mi = lij[m * n + i];
            axx += k_mi * (1.0 - l_mi * dy * dy / den);
            axy += k_mi * l_mi * dx * dy / den;
            ayy += k_mi * (1.0 - l_mi * dx * dx / den);
        }

        let ax = -d1[m];
        let ay = -d2[m];

        // Solve 2×2: [Axx Axy; Axy Ayy] * [dx; dy] = [Ax; Ay]
        let (delta_x, delta_y) = if ax * ax + ay * ay < KK_EPS * KK_EPS {
            (0.0, 0.0)
        } else {
            let det = axx * ayy - axy * axy;
            if det.abs() < f64::EPSILON {
                (0.0, 0.0)
            } else {
                ((ax * ayy - ay * axy) / det, (axx * ay - axy * ax) / det)
            }
        };

        let mut new_x = old_x + delta_x;
        let mut new_y = old_y + delta_y;

        // Apply bounds
        if let Some(b) = bounds {
            if let Some(ref minx) = b.minx {
                if new_x < minx[m] {
                    new_x = minx[m];
                }
            }
            if let Some(ref maxx) = b.maxx {
                if new_x > maxx[m] {
                    new_x = maxx[m];
                }
            }
            if let Some(ref miny) = b.miny {
                if new_y < miny[m] {
                    new_y = miny[m];
                }
            }
            if let Some(ref maxy) = b.maxy {
                if new_y > maxy[m] {
                    new_y = maxy[m];
                }
            }
        }

        // Update gradient incrementally
        d1[m] = 0.0;
        d2[m] = 0.0;
        for i in 0..n {
            if i == m {
                continue;
            }
            let old_dx = old_x - pos[i][0];
            let old_dy = old_y - pos[i][1];
            let old_dist = (old_dx * old_dx + old_dy * old_dy).sqrt();
            let new_dx = new_x - pos[i][0];
            let new_dy = new_y - pos[i][1];
            let new_dist = (new_dx * new_dx + new_dy * new_dy).sqrt();

            let k_mi = kij[m * n + i];
            let l_mi = lij[m * n + i];

            // Remove old contribution to i's gradient, add new
            d1[i] -= k_mi * (-old_dx + l_mi * old_dx / old_dist);
            d2[i] -= k_mi * (-old_dy + l_mi * old_dy / old_dist);
            d1[i] += k_mi * (-new_dx + l_mi * new_dx / new_dist);
            d2[i] += k_mi * (-new_dy + l_mi * new_dy / new_dist);

            // Accumulate m's new gradient
            d1[m] += k_mi * (new_dx - l_mi * new_dx / new_dist);
            d2[m] += k_mi * (new_dy - l_mi * new_dy / new_dist);
        }

        pos[m] = [new_x, new_y];
    }

    Ok(pos)
}

/// Compute 3D Kamada-Kawai layout.
///
/// Three-dimensional variant; same algorithm but solves a 3×3 system
/// per iteration using Cramer's rule.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_kamada_kawai_3d, KkParams3d};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 0).unwrap();
///
/// let params = KkParams3d::default_for(4);
/// let pos = layout_kamada_kawai_3d(&g, None, &params, None).unwrap();
/// assert_eq!(pos.len(), 4);
/// assert_eq!(pos[0].len(), 3); // x, y, z
/// ```
pub fn layout_kamada_kawai_3d(
    graph: &Graph,
    weights: Option<&[f64]>,
    params: &KkParams3d,
    bounds: Option<&KkBounds3d>,
) -> IgraphResult<Vec<[f64; 3]>> {
    let n = graph.vcount() as usize;

    if params.kkconst <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "kkconst must be positive".into(),
        ));
    }
    if let Some(ws) = weights {
        if ws.len() != graph.ecount() {
            return Err(IgraphError::InvalidArgument(
                "weights length must equal edge count".into(),
            ));
        }
        if graph.ecount() > 0 {
            for &w in ws {
                if w <= 0.0 || w.is_nan() {
                    return Err(IgraphError::InvalidArgument(
                        "weights must be positive for Kamada-Kawai layout".into(),
                    ));
                }
            }
        }
    }

    validate_bounds_3d(n, bounds)?;

    let mut pos = match &params.seed {
        Some(seed) => {
            if seed.len() != n {
                return Err(IgraphError::InvalidArgument(
                    "seed length must equal vertex count".into(),
                ));
            }
            seed.clone()
        }
        None => initial_sphere_3d(n),
    };

    if n <= 1 {
        return Ok(pos);
    }

    let mut dij = all_pairs_distances(graph, weights)?;
    let max_dij = find_max_finite(&mut dij, n);
    if max_dij == 0.0 {
        return Ok(pos);
    }

    let l0 = (n as f64).sqrt();
    let big_l = l0 / max_dij;

    let mut kij = vec![0.0_f64; n * n];
    let mut lij = vec![0.0_f64; n * n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let d = dij[i * n + j];
            let d2 = d * d;
            if d2 > 0.0 {
                kij[i * n + j] = params.kkconst / d2;
                lij[i * n + j] = big_l * d;
            }
        }
    }

    // Initialize gradients
    let mut d1 = vec![0.0_f64; n];
    let mut d2 = vec![0.0_f64; n];
    let mut d3 = vec![0.0_f64; n];
    for m in 0..n {
        let (gx, gy, gz) = gradient_3d(m, &pos, &kij, &lij, n);
        d1[m] = gx;
        d2[m] = gy;
        d3[m] = gz;
    }

    for _iter in 0..params.maxiter {
        let (m, max_delta) = select_max_delta_3d(&d1, &d2, &d3, n);
        if max_delta < params.epsilon {
            break;
        }

        let old_x = pos[m][0];
        let old_y = pos[m][1];
        let old_z = pos[m][2];

        // Build 3×3 Hessian
        let mut axx = 0.0_f64;
        let mut axy = 0.0_f64;
        let mut axz = 0.0_f64;
        let mut ayy = 0.0_f64;
        let mut ayz = 0.0_f64;
        let mut azz = 0.0_f64;

        for i in 0..n {
            if i == m {
                continue;
            }
            let dx = old_x - pos[i][0];
            let dy = old_y - pos[i][1];
            let dz = old_z - pos[i][2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            let den = dist * (dx * dx + dy * dy + dz * dz);
            let k_mi = kij[m * n + i];
            let l_mi = lij[m * n + i];

            axx += k_mi * (1.0 - l_mi * (dy * dy + dz * dz) / den);
            ayy += k_mi * (1.0 - l_mi * (dx * dx + dz * dz) / den);
            azz += k_mi * (1.0 - l_mi * (dx * dx + dy * dy) / den);
            axy += k_mi * l_mi * dx * dy / den;
            axz += k_mi * l_mi * dx * dz / den;
            ayz += k_mi * l_mi * dy * dz / den;
        }

        let ax = -d1[m];
        let ay = -d2[m];
        let az = -d3[m];

        // Solve 3×3 via Cramer's rule
        let (delta_x, delta_y, delta_z) = if ax * ax + ay * ay + az * az < KK3D_EPS * KK3D_EPS {
            (0.0, 0.0, 0.0)
        } else {
            let detnum = det3(axx, axy, axz, axy, ayy, ayz, axz, ayz, azz);
            if detnum.abs() < f64::EPSILON {
                (0.0, 0.0, 0.0)
            } else {
                let dx = det3(ax, ay, az, axy, ayy, ayz, axz, ayz, azz) / detnum;
                let dy = det3(axx, axy, axz, ax, ay, az, axz, ayz, azz) / detnum;
                let dz = det3(axx, axy, axz, axy, ayy, ayz, ax, ay, az) / detnum;
                (dx, dy, dz)
            }
        };

        let mut new_x = old_x + delta_x;
        let mut new_y = old_y + delta_y;
        let mut new_z = old_z + delta_z;

        // Apply bounds
        if let Some(b) = bounds {
            if let Some(ref v) = b.minx {
                if new_x < v[m] {
                    new_x = v[m];
                }
            }
            if let Some(ref v) = b.maxx {
                if new_x > v[m] {
                    new_x = v[m];
                }
            }
            if let Some(ref v) = b.miny {
                if new_y < v[m] {
                    new_y = v[m];
                }
            }
            if let Some(ref v) = b.maxy {
                if new_y > v[m] {
                    new_y = v[m];
                }
            }
            if let Some(ref v) = b.minz {
                if new_z < v[m] {
                    new_z = v[m];
                }
            }
            if let Some(ref v) = b.maxz {
                if new_z > v[m] {
                    new_z = v[m];
                }
            }
        }

        // Incremental gradient update
        d1[m] = 0.0;
        d2[m] = 0.0;
        d3[m] = 0.0;
        for i in 0..n {
            if i == m {
                continue;
            }
            let old_dx = old_x - pos[i][0];
            let old_dy = old_y - pos[i][1];
            let old_dz = old_z - pos[i][2];
            let old_dist = (old_dx * old_dx + old_dy * old_dy + old_dz * old_dz).sqrt();
            let new_dx = new_x - pos[i][0];
            let new_dy = new_y - pos[i][1];
            let new_dz = new_z - pos[i][2];
            let new_dist = (new_dx * new_dx + new_dy * new_dy + new_dz * new_dz).sqrt();

            let k_mi = kij[m * n + i];
            let l_mi = lij[m * n + i];

            d1[i] -= k_mi * (-old_dx + l_mi * old_dx / old_dist);
            d2[i] -= k_mi * (-old_dy + l_mi * old_dy / old_dist);
            d3[i] -= k_mi * (-old_dz + l_mi * old_dz / old_dist);
            d1[i] += k_mi * (-new_dx + l_mi * new_dx / new_dist);
            d2[i] += k_mi * (-new_dy + l_mi * new_dy / new_dist);
            d3[i] += k_mi * (-new_dz + l_mi * new_dz / new_dist);

            d1[m] += k_mi * (new_dx - l_mi * new_dx / new_dist);
            d2[m] += k_mi * (new_dy - l_mi * new_dy / new_dist);
            d3[m] += k_mi * (new_dz - l_mi * new_dz / new_dist);
        }

        pos[m] = [new_x, new_y, new_z];
    }

    Ok(pos)
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

impl KkParams {
    /// Reasonable defaults for a graph with `n` vertices.
    pub fn default_for(n: usize) -> Self {
        Self {
            maxiter: 10 * n.max(1),
            epsilon: 0.0,
            kkconst: n as f64,
            seed: None,
        }
    }
}

impl KkParams3d {
    /// Reasonable defaults for a graph with `n` vertices.
    pub fn default_for(n: usize) -> Self {
        Self {
            maxiter: 10 * n.max(1),
            epsilon: 0.0,
            kkconst: n as f64,
            seed: None,
        }
    }
}

fn initial_circle_2d(n: usize) -> Vec<[f64; 2]> {
    let l0 = (n as f64).sqrt();
    let scale = 0.36 * l0;
    let mut pos = Vec::with_capacity(n);
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        pos.push([scale * angle.cos(), scale * angle.sin()]);
    }
    pos
}

fn initial_sphere_3d(n: usize) -> Vec<[f64; 3]> {
    let l0 = (n as f64).sqrt();
    let scale = 0.36 * l0;
    let mut pos = Vec::with_capacity(n);
    let golden_ratio = 0.5 * (1.0 + 5.0_f64.sqrt());
    for i in 0..n {
        let theta = 2.0 * std::f64::consts::PI * (i as f64) / golden_ratio;
        let phi = (1.0 - 2.0 * (i as f64 + 0.5) / n as f64).acos();
        pos.push([
            scale * phi.sin() * theta.cos(),
            scale * phi.sin() * theta.sin(),
            scale * phi.cos(),
        ]);
    }
    pos
}

fn initial_bounded_2d(n: usize, bounds: Option<&KkBounds>) -> Vec<[f64; 2]> {
    let mut pos = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / n.max(1) as f64;
        let angle = 2.0 * std::f64::consts::PI * t;
        let (mut x, mut y) = (angle.cos(), angle.sin());
        // Scale into bounding box if provided
        if let Some(b) = bounds {
            let (lo_x, hi_x) = bounds_range_x(b, i);
            let (lo_y, hi_y) = bounds_range_y(b, i);
            x = lo_x + (x + 1.0) * 0.5 * (hi_x - lo_x);
            y = lo_y + (y + 1.0) * 0.5 * (hi_y - lo_y);
        }
        pos.push([x, y]);
    }
    pos
}

fn bounds_range_x(b: &KkBounds, i: usize) -> (f64, f64) {
    let lo = b.minx.as_ref().map_or(-1.0, |v| v[i]);
    let hi = b.maxx.as_ref().map_or(1.0, |v| v[i]);
    (lo, hi)
}

fn bounds_range_y(b: &KkBounds, i: usize) -> (f64, f64) {
    let lo = b.miny.as_ref().map_or(-1.0, |v| v[i]);
    let hi = b.maxy.as_ref().map_or(1.0, |v| v[i]);
    (lo, hi)
}

fn clamp_positions_2d(pos: &mut [[f64; 2]], bounds: &KkBounds) {
    for (i, p) in pos.iter_mut().enumerate() {
        if let Some(ref v) = bounds.minx {
            if p[0] < v[i] {
                p[0] = v[i];
            }
        }
        if let Some(ref v) = bounds.maxx {
            if p[0] > v[i] {
                p[0] = v[i];
            }
        }
        if let Some(ref v) = bounds.miny {
            if p[1] < v[i] {
                p[1] = v[i];
            }
        }
        if let Some(ref v) = bounds.maxy {
            if p[1] > v[i] {
                p[1] = v[i];
            }
        }
    }
}

fn all_pairs_distances(graph: &Graph, weights: Option<&[f64]>) -> IgraphResult<Vec<f64>> {
    let n = graph.vcount() as usize;
    let dist_matrix = floyd_warshall_distances(graph, weights)?;

    let mut flat = vec![f64::INFINITY; n * n];
    for i in 0..n {
        for j in 0..n {
            if let Some(d) = dist_matrix[i][j] {
                flat[i * n + j] = d;
            }
        }
    }
    Ok(flat)
}

/// Find max finite distance and replace all infinite entries with it.
fn find_max_finite(dij: &mut [f64], n: usize) -> f64 {
    let mut max_d = 0.0_f64;
    for i in 0..n {
        for j in (i + 1)..n {
            let d = dij[i * n + j];
            if d.is_finite() && d > max_d {
                max_d = d;
            }
        }
    }
    // Replace infinite distances with max_dij (disconnected components)
    for val in dij.iter_mut() {
        if !val.is_finite() {
            *val = max_d;
        }
    }
    max_d
}

fn gradient_2d(m: usize, pos: &[[f64; 2]], kij: &[f64], lij: &[f64], n: usize) -> (f64, f64) {
    let mut gx = 0.0;
    let mut gy = 0.0;
    for i in 0..n {
        if i == m {
            continue;
        }
        let dx = pos[m][0] - pos[i][0];
        let dy = pos[m][1] - pos[i][1];
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < f64::EPSILON {
            continue;
        }
        let k = kij[m * n + i];
        let l = lij[m * n + i];
        gx += k * (dx - l * dx / dist);
        gy += k * (dy - l * dy / dist);
    }
    (gx, gy)
}

fn gradient_3d(m: usize, pos: &[[f64; 3]], kij: &[f64], lij: &[f64], n: usize) -> (f64, f64, f64) {
    let mut gx = 0.0;
    let mut gy = 0.0;
    let mut gz = 0.0;
    for i in 0..n {
        if i == m {
            continue;
        }
        let dx = pos[m][0] - pos[i][0];
        let dy = pos[m][1] - pos[i][1];
        let dz = pos[m][2] - pos[i][2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < f64::EPSILON {
            continue;
        }
        let k = kij[m * n + i];
        let l = lij[m * n + i];
        gx += k * (dx - l * dx / dist);
        gy += k * (dy - l * dy / dist);
        gz += k * (dz - l * dz / dist);
    }
    (gx, gy, gz)
}

fn select_max_delta_2d(d1: &[f64], d2: &[f64], n: usize) -> (usize, f64) {
    let mut m = 0;
    let mut max_delta = -1.0_f64;
    for i in 0..n {
        let delta = d1[i] * d1[i] + d2[i] * d2[i];
        if delta > max_delta {
            m = i;
            max_delta = delta;
        }
    }
    (m, max_delta)
}

fn select_max_delta_3d(d1: &[f64], d2: &[f64], d3: &[f64], n: usize) -> (usize, f64) {
    let mut m = 0;
    let mut max_delta = -1.0_f64;
    for i in 0..n {
        let delta = d1[i] * d1[i] + d2[i] * d2[i] + d3[i] * d3[i];
        if delta > max_delta {
            m = i;
            max_delta = delta;
        }
    }
    (m, max_delta)
}

/// 3×3 determinant via Sarrus' rule.
#[allow(clippy::too_many_arguments, clippy::many_single_char_names)]
fn det3(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: f64, i: f64) -> f64 {
    a * e * i + b * f * g + c * d * h - c * e * g - b * d * i - a * f * h
}

fn validate_bounds_2d(n: usize, bounds: Option<&KkBounds>) -> IgraphResult<()> {
    let Some(b) = bounds else { return Ok(()) };
    if let Some(ref v) = b.minx {
        if v.len() != n {
            return Err(IgraphError::InvalidArgument("minx length mismatch".into()));
        }
    }
    if let Some(ref v) = b.maxx {
        if v.len() != n {
            return Err(IgraphError::InvalidArgument("maxx length mismatch".into()));
        }
    }
    if let Some(ref v) = b.miny {
        if v.len() != n {
            return Err(IgraphError::InvalidArgument("miny length mismatch".into()));
        }
    }
    if let Some(ref v) = b.maxy {
        if v.len() != n {
            return Err(IgraphError::InvalidArgument("maxy length mismatch".into()));
        }
    }
    if let (Some(lo), Some(hi)) = (&b.minx, &b.maxx) {
        for i in 0..n {
            if lo[i] > hi[i] {
                return Err(IgraphError::InvalidArgument(
                    "minx must not exceed maxx".into(),
                ));
            }
        }
    }
    if let (Some(lo), Some(hi)) = (&b.miny, &b.maxy) {
        for i in 0..n {
            if lo[i] > hi[i] {
                return Err(IgraphError::InvalidArgument(
                    "miny must not exceed maxy".into(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_bounds_3d(n: usize, bounds: Option<&KkBounds3d>) -> IgraphResult<()> {
    let Some(b) = bounds else { return Ok(()) };
    for (name, v) in [
        ("minx", &b.minx),
        ("maxx", &b.maxx),
        ("miny", &b.miny),
        ("maxy", &b.maxy),
        ("minz", &b.minz),
        ("maxz", &b.maxz),
    ] {
        if let Some(vec) = v {
            if vec.len() != n {
                return Err(IgraphError::InvalidArgument(format!(
                    "{name} length mismatch"
                )));
            }
        }
    }
    if let (Some(lo), Some(hi)) = (&b.minx, &b.maxx) {
        for i in 0..n {
            if lo[i] > hi[i] {
                return Err(IgraphError::InvalidArgument(
                    "minx must not exceed maxx".into(),
                ));
            }
        }
    }
    if let (Some(lo), Some(hi)) = (&b.miny, &b.maxy) {
        for i in 0..n {
            if lo[i] > hi[i] {
                return Err(IgraphError::InvalidArgument(
                    "miny must not exceed maxy".into(),
                ));
            }
        }
    }
    if let (Some(lo), Some(hi)) = (&b.minz, &b.maxz) {
        for i in 0..n {
            if lo[i] > hi[i] {
                return Err(IgraphError::InvalidArgument(
                    "minz must not exceed maxz".into(),
                ));
            }
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::VertexId;

    fn path_graph(n: usize) -> Graph {
        let mut g = Graph::with_vertices(n as VertexId);
        for i in 0..(n - 1) {
            g.add_edge(i as VertexId, (i + 1) as VertexId).unwrap();
        }
        g
    }

    fn cycle_graph(n: usize) -> Graph {
        let mut g = Graph::with_vertices(n as VertexId);
        for i in 0..n {
            g.add_edge(i as VertexId, ((i + 1) % n) as VertexId)
                .unwrap();
        }
        g
    }

    #[test]
    fn test_kk_single_vertex() {
        let g = Graph::with_vertices(1);
        let params = KkParams::default_for(1);
        let pos = layout_kamada_kawai(&g, None, &params, None).unwrap();
        assert_eq!(pos.len(), 1);
    }

    #[test]
    fn test_kk_two_vertices() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let params = KkParams::default_for(2);
        let pos = layout_kamada_kawai(&g, None, &params, None).unwrap();
        assert_eq!(pos.len(), 2);
        // Should be separated
        let dist = ((pos[0][0] - pos[1][0]).powi(2) + (pos[0][1] - pos[1][1]).powi(2)).sqrt();
        assert!(dist > 0.01, "vertices should be separated, got dist={dist}");
    }

    #[test]
    fn test_kk_path_maintains_order() {
        let g = path_graph(5);
        let params = KkParams {
            maxiter: 200,
            epsilon: 1e-4,
            kkconst: 5.0,
            seed: None,
        };
        let pos = layout_kamada_kawai(&g, None, &params, None).unwrap();
        assert_eq!(pos.len(), 5);
        // All positions should be finite
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_kk_cycle_symmetric() {
        let g = cycle_graph(6);
        let params = KkParams {
            maxiter: 300,
            epsilon: 1e-6,
            kkconst: 6.0,
            seed: None,
        };
        let pos = layout_kamada_kawai(&g, None, &params, None).unwrap();
        // Centroid should be near origin
        let cx: f64 = pos.iter().map(|p| p[0]).sum::<f64>() / 6.0;
        let cy: f64 = pos.iter().map(|p| p[1]).sum::<f64>() / 6.0;
        assert!(cx.abs() < 0.5, "centroid x={cx}");
        assert!(cy.abs() < 0.5, "centroid y={cy}");
    }

    #[test]
    fn test_kk_weighted() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();
        let weights = [1.0, 1.0, 5.0]; // 0-2 edge is "long"
        let params = KkParams {
            maxiter: 200,
            epsilon: 1e-6,
            kkconst: 3.0,
            seed: None,
        };
        let pos = layout_kamada_kawai(&g, Some(&weights), &params, None).unwrap();
        // Distance 0→2 via direct edge (weight 5) vs 0→1→2 (weight 2)
        // Shortest path 0→2 is min(5, 2)=2, so 0-2 should be closer than weight 5 suggests
        let d01 = ((pos[0][0] - pos[1][0]).powi(2) + (pos[0][1] - pos[1][1]).powi(2)).sqrt();
        let d02 = ((pos[0][0] - pos[2][0]).powi(2) + (pos[0][1] - pos[2][1]).powi(2)).sqrt();
        // d02 should be roughly twice d01 (path distance 2 vs 1)
        assert!(d02 > d01 * 0.5, "d02={d02}, d01={d01}");
        assert!(pos[0][0].is_finite());
    }

    #[test]
    fn test_kk_bounds() {
        let g = path_graph(3);
        let params = KkParams::default_for(3);
        let bounds = KkBounds {
            minx: Some(vec![0.0, 0.0, 0.0]),
            maxx: Some(vec![1.0, 1.0, 1.0]),
            miny: Some(vec![0.0, 0.0, 0.0]),
            maxy: Some(vec![1.0, 1.0, 1.0]),
        };
        let pos = layout_kamada_kawai(&g, None, &params, Some(&bounds)).unwrap();
        for p in &pos {
            assert!(p[0] >= 0.0 && p[0] <= 1.0, "x={} out of bounds", p[0]);
            assert!(p[1] >= 0.0 && p[1] <= 1.0, "y={} out of bounds", p[1]);
        }
    }

    #[test]
    fn test_kk_invalid_weights() {
        let mut g = Graph::with_vertices(2);
        g.add_edge(0, 1).unwrap();
        let params = KkParams::default_for(2);
        let result = layout_kamada_kawai(&g, Some(&[-1.0]), &params, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_kk_invalid_kkconst() {
        let g = Graph::with_vertices(2);
        let params = KkParams {
            maxiter: 10,
            epsilon: 0.0,
            kkconst: 0.0,
            seed: None,
        };
        let result = layout_kamada_kawai(&g, None, &params, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_kk_3d_basic() {
        let g = cycle_graph(4);
        let params = KkParams3d::default_for(4);
        let pos = layout_kamada_kawai_3d(&g, None, &params, None).unwrap();
        assert_eq!(pos.len(), 4);
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
            assert!(p[2].is_finite());
        }
    }

    #[test]
    fn test_kk_3d_bounds() {
        let g = path_graph(3);
        let params = KkParams3d::default_for(3);
        let bounds = KkBounds3d {
            minx: Some(vec![-1.0, -1.0, -1.0]),
            maxx: Some(vec![1.0, 1.0, 1.0]),
            miny: Some(vec![-1.0, -1.0, -1.0]),
            maxy: Some(vec![1.0, 1.0, 1.0]),
            minz: Some(vec![-1.0, -1.0, -1.0]),
            maxz: Some(vec![1.0, 1.0, 1.0]),
        };
        let pos = layout_kamada_kawai_3d(&g, None, &params, Some(&bounds)).unwrap();
        for p in &pos {
            assert!(p[0] >= -1.0 && p[0] <= 1.0);
            assert!(p[1] >= -1.0 && p[1] <= 1.0);
            assert!(p[2] >= -1.0 && p[2] <= 1.0);
        }
    }

    #[test]
    fn test_kk_seed() {
        let g = path_graph(3);
        let seed = vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let params = KkParams {
            maxiter: 100,
            epsilon: 1e-6,
            kkconst: 3.0,
            seed: Some(seed),
        };
        let pos = layout_kamada_kawai(&g, None, &params, None).unwrap();
        assert_eq!(pos.len(), 3);
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }

    #[test]
    fn test_kk_disconnected() {
        // Two isolated components
        let mut g = Graph::with_vertices(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        let params = KkParams::default_for(4);
        let pos = layout_kamada_kawai(&g, None, &params, None).unwrap();
        assert_eq!(pos.len(), 4);
        // All should be finite — disconnected vertices get max_dij distance
        for p in &pos {
            assert!(p[0].is_finite());
            assert!(p[1].is_finite());
        }
    }
}
