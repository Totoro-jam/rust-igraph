//! Simple graph layout algorithms (ALGO-LO-001).
//!
//! Counterparts of `igraph_layout_random`, `igraph_layout_circle`,
//! `igraph_layout_star`, `igraph_layout_grid`, `igraph_layout_sphere`,
//! and their 3D variants from `references/igraph/src/layout/`.

use std::f64::consts::PI;

use crate::core::{Graph, IgraphResult, rng::SplitMix64};

/// Place vertices uniformly at random in the square `[-1, 1] × [-1, 1]`.
///
/// Returns a `Vec` of `(x, y)` coordinate pairs, one per vertex.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_random};
///
/// let g = Graph::with_vertices(5);
/// let coords = layout_random(&g, 42);
/// assert_eq!(coords.len(), 5);
/// for &(x, y) in &coords {
///     assert!((-1.0..=1.0).contains(&x));
///     assert!((-1.0..=1.0).contains(&y));
/// }
/// ```
pub fn layout_random(graph: &Graph, seed: u64) -> Vec<(f64, f64)> {
    let n = graph.vcount() as usize;
    let mut rng = SplitMix64::new(seed);
    (0..n)
        .map(|_| {
            let x = 2.0 * rng.gen_unit() - 1.0;
            let y = 2.0 * rng.gen_unit() - 1.0;
            (x, y)
        })
        .collect()
}

/// Place vertices uniformly at random in the cube `[-1, 1]³`.
///
/// Returns a `Vec` of `(x, y, z)` coordinate triples, one per vertex.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_random_3d};
///
/// let g = Graph::with_vertices(5);
/// let coords = layout_random_3d(&g, 42);
/// assert_eq!(coords.len(), 5);
/// ```
pub fn layout_random_3d(graph: &Graph, seed: u64) -> Vec<(f64, f64, f64)> {
    let n = graph.vcount() as usize;
    let mut rng = SplitMix64::new(seed);
    (0..n)
        .map(|_| {
            let x = 2.0 * rng.gen_unit() - 1.0;
            let y = 2.0 * rng.gen_unit() - 1.0;
            let z = 2.0 * rng.gen_unit() - 1.0;
            (x, y, z)
        })
        .collect()
}

/// Place vertices uniformly on a unit circle.
///
/// Vertices are placed in the order given by `order` (if `Some`), or
/// in vertex-ID order. Vertices not in `order` are placed at `(0, 0)`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_circle};
///
/// let g = Graph::with_vertices(4);
/// let coords = layout_circle(&g, None);
/// assert_eq!(coords.len(), 4);
/// // First vertex is at (1, 0)
/// assert!((coords[0].0 - 1.0).abs() < 1e-9);
/// assert!(coords[0].1.abs() < 1e-9);
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn layout_circle(graph: &Graph, order: Option<&[u32]>) -> Vec<(f64, f64)> {
    let n = graph.vcount() as usize;
    let mut coords = vec![(0.0, 0.0); n];

    match order {
        Some(ord) => {
            let vs_size = ord.len();
            for (i, &v) in ord.iter().enumerate() {
                let v_us = v as usize;
                if v_us < n {
                    let phi = 2.0 * PI / (vs_size as f64) * (i as f64);
                    coords[v_us] = (phi.cos(), phi.sin());
                }
            }
        }
        None => {
            for (i, slot) in coords.iter_mut().enumerate() {
                let phi = 2.0 * PI / (n as f64) * (i as f64);
                *slot = (phi.cos(), phi.sin());
            }
        }
    }

    coords
}

/// Place vertices in a star layout: center at origin, rest on a unit
/// circle.
///
/// # Parameters
///
/// - `center`: vertex id to place at the origin.
/// - `order`: optional ordering of vertices. If `None`, vertices are
///   placed in vertex-ID order.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_star};
///
/// let g = Graph::with_vertices(5);
/// let coords = layout_star(&g, 0, None).unwrap();
/// assert_eq!(coords.len(), 5);
/// // Center vertex is at origin
/// assert!((coords[0].0).abs() < 1e-9);
/// assert!((coords[0].1).abs() < 1e-9);
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn layout_star(
    graph: &Graph,
    center: u32,
    order: Option<&[u32]>,
) -> IgraphResult<Vec<(f64, f64)>> {
    let n = graph.vcount();
    let n_us = n as usize;

    if n > 0 && center >= n {
        return Err(crate::core::IgraphError::InvalidArgument(
            "center vertex out of range".to_string(),
        ));
    }
    if let Some(ord) = order {
        if ord.len() != n_us {
            return Err(crate::core::IgraphError::InvalidArgument(
                "order vector length must equal vertex count".to_string(),
            ));
        }
    }

    let mut coords = vec![(0.0, 0.0); n_us];

    if n_us <= 1 {
        return Ok(coords);
    }

    let ring_count = n_us - 1;
    let mut phi = 0.0_f64;

    for i in 0..n_us {
        #[allow(clippy::cast_possible_truncation)]
        let node = order.map_or(i as u32, |ord| ord[i]);
        if node != center {
            coords[node as usize] = (phi.cos(), phi.sin());
            phi += 2.0 * PI / (ring_count as f64);
        }
    }

    Ok(coords)
}

/// Place vertices on a regular 2D grid.
///
/// When `width` is 0 or negative, the grid width is
/// `ceil(sqrt(vcount))`.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_grid};
///
/// let g = Graph::with_vertices(6);
/// let coords = layout_grid(&g, 3);
/// assert_eq!(coords.len(), 6);
/// // First row: (0,0), (1,0), (2,0)
/// assert!((coords[0].0).abs() < 1e-9);
/// assert!((coords[1].0 - 1.0).abs() < 1e-9);
/// assert!((coords[2].0 - 2.0).abs() < 1e-9);
/// // Second row: (0,1), (1,1), (2,1)
/// assert!((coords[3].1 - 1.0).abs() < 1e-9);
/// ```
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn layout_grid(graph: &Graph, width: i32) -> Vec<(f64, f64)> {
    let n = graph.vcount() as usize;

    let w = if width <= 0 {
        (n as f64).sqrt().ceil() as usize
    } else {
        width as usize
    };

    let mut coords = Vec::with_capacity(n);
    let mut x = 0usize;
    let mut y = 0usize;

    for _ in 0..n {
        coords.push((x as f64, y as f64));
        x += 1;
        if x == w {
            x = 0;
            y += 1;
        }
    }

    coords
}

/// Place vertices on a regular 3D grid.
///
/// When `width` or `height` is 0 or negative, it is auto-determined.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_grid_3d};
///
/// let g = Graph::with_vertices(8);
/// let coords = layout_grid_3d(&g, 2, 2);
/// assert_eq!(coords.len(), 8);
/// ```
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]
pub fn layout_grid_3d(graph: &Graph, width: i32, height: i32) -> Vec<(f64, f64, f64)> {
    let n = graph.vcount() as usize;

    let (w, h) = match (width <= 0, height <= 0) {
        (true, true) => {
            let side = (n as f64).cbrt().ceil() as usize;
            (side.max(1), side.max(1))
        }
        (true, false) => {
            let ht = height as usize;
            let wd = ((n as f64) / (ht as f64)).sqrt().ceil() as usize;
            (wd.max(1), ht)
        }
        (false, true) => {
            let wd = width as usize;
            let ht = ((n as f64) / (wd as f64)).sqrt().ceil() as usize;
            (wd, ht.max(1))
        }
        (false, false) => (width as usize, height as usize),
    };

    let mut coords = Vec::with_capacity(n);
    let mut x = 0usize;
    let mut y = 0usize;
    let mut z = 0usize;

    for _ in 0..n {
        coords.push((x as f64, y as f64, z as f64));
        x += 1;
        if x == w {
            x = 0;
            y += 1;
            if y == h {
                y = 0;
                z += 1;
            }
        }
    }

    coords
}

/// Place vertices approximately uniformly on a unit sphere.
///
/// Uses the Saff-Kuijlaars spiral algorithm (Mathematical
/// Intelligencer 19.1, 1997).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_sphere};
///
/// let g = Graph::with_vertices(10);
/// let coords = layout_sphere(&g);
/// assert_eq!(coords.len(), 10);
/// // All points are approximately on the unit sphere
/// for &(x, y, z) in &coords {
///     let r = (x * x + y * y + z * z).sqrt();
///     assert!((r - 1.0).abs() < 0.01);
/// }
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn layout_sphere(graph: &Graph) -> Vec<(f64, f64, f64)> {
    let n = graph.vcount() as usize;

    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![(0.0, 0.0, -1.0)];
    }

    let sqrt_n = (n as f64).sqrt();
    let mut coords = Vec::with_capacity(n);
    let mut phi = 0.0_f64;

    for i in 0..n {
        let (z, r) = if i == 0 {
            (-1.0, 0.0)
        } else if i == n - 1 {
            (1.0, 0.0)
        } else {
            let z_val = -1.0 + 2.0 * (i as f64) / ((n - 1) as f64);
            let r_val = (1.0 - z_val * z_val).sqrt();
            phi += 3.6 / (sqrt_n * r_val);
            (z_val, r_val)
        };

        coords.push((r * phi.cos(), r * phi.sin(), z));
    }

    coords
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn random_empty() {
        let g = Graph::with_vertices(0);
        let c = layout_random(&g, 0);
        assert!(c.is_empty());
    }

    #[test]
    fn random_bounds() {
        let g = Graph::with_vertices(100);
        let c = layout_random(&g, 42);
        assert_eq!(c.len(), 100);
        for &(x, y) in &c {
            assert!((-1.0..=1.0).contains(&x), "x={x} out of bounds");
            assert!((-1.0..=1.0).contains(&y), "y={y} out of bounds");
        }
    }

    #[test]
    fn random_deterministic() {
        let g = Graph::with_vertices(10);
        let a = layout_random(&g, 123);
        let b = layout_random(&g, 123);
        assert_eq!(a, b);
    }

    #[test]
    fn random_3d_bounds() {
        let g = Graph::with_vertices(50);
        let c = layout_random_3d(&g, 42);
        assert_eq!(c.len(), 50);
        for &(x, y, z) in &c {
            assert!((-1.0..=1.0).contains(&x));
            assert!((-1.0..=1.0).contains(&y));
            assert!((-1.0..=1.0).contains(&z));
        }
    }

    #[test]
    fn circle_empty() {
        let g = Graph::with_vertices(0);
        let c = layout_circle(&g, None);
        assert!(c.is_empty());
    }

    #[test]
    fn circle_single() {
        let g = Graph::with_vertices(1);
        let c = layout_circle(&g, None);
        assert_eq!(c.len(), 1);
        assert!(approx_eq(c[0].0, 1.0));
        assert!(approx_eq(c[0].1, 0.0));
    }

    #[test]
    fn circle_4_vertices() {
        let g = Graph::with_vertices(4);
        let c = layout_circle(&g, None);
        assert_eq!(c.len(), 4);
        assert!(approx_eq(c[0].0, 1.0));
        assert!(approx_eq(c[0].1, 0.0));
        assert!(approx_eq(c[1].0, 0.0));
        assert!(approx_eq(c[1].1, 1.0));
        assert!(approx_eq(c[2].0, -1.0));
        assert!(approx_eq(c[2].1, 0.0));
        assert!(approx_eq(c[3].0, 0.0));
        assert!(approx_eq(c[3].1, -1.0));
    }

    #[test]
    fn circle_with_order() {
        let g = Graph::with_vertices(3);
        let c = layout_circle(&g, Some(&[2, 0, 1]));
        assert!(approx_eq(c[2].0, 1.0));
        assert!(approx_eq(c[2].1, 0.0));
    }

    #[test]
    fn circle_unit_radius() {
        let g = Graph::with_vertices(12);
        let c = layout_circle(&g, None);
        for (i, &(x, y)) in c.iter().enumerate() {
            let r = (x * x + y * y).sqrt();
            assert!(approx_eq(r, 1.0), "v{i}: r={r}");
        }
    }

    #[test]
    fn star_empty() {
        let g = Graph::with_vertices(0);
        let c = layout_star(&g, 0, None).expect("ok");
        assert!(c.is_empty());
    }

    #[test]
    fn star_single() {
        let g = Graph::with_vertices(1);
        let c = layout_star(&g, 0, None).expect("ok");
        assert_eq!(c.len(), 1);
        assert!(approx_eq(c[0].0, 0.0));
        assert!(approx_eq(c[0].1, 0.0));
    }

    #[test]
    fn star_center_at_origin() {
        let g = Graph::with_vertices(5);
        let c = layout_star(&g, 2, None).expect("ok");
        assert!(approx_eq(c[2].0, 0.0));
        assert!(approx_eq(c[2].1, 0.0));
    }

    #[test]
    fn star_ring_unit_radius() {
        let g = Graph::with_vertices(5);
        let c = layout_star(&g, 0, None).expect("ok");
        for (i, &(x, y)) in c.iter().enumerate().skip(1) {
            let r = (x * x + y * y).sqrt();
            assert!(approx_eq(r, 1.0), "v{i}: r={r}");
        }
    }

    #[test]
    fn star_invalid_center() {
        let g = Graph::with_vertices(3);
        let r = layout_star(&g, 5, None);
        assert!(r.is_err());
    }

    #[test]
    fn star_invalid_order_len() {
        let g = Graph::with_vertices(3);
        let r = layout_star(&g, 0, Some(&[0, 1]));
        assert!(r.is_err());
    }

    #[test]
    fn grid_empty() {
        let g = Graph::with_vertices(0);
        let c = layout_grid(&g, 3);
        assert!(c.is_empty());
    }

    #[test]
    fn grid_2x3() {
        let g = Graph::with_vertices(6);
        let c = layout_grid(&g, 3);
        assert_eq!(c[0], (0.0, 0.0));
        assert_eq!(c[1], (1.0, 0.0));
        assert_eq!(c[2], (2.0, 0.0));
        assert_eq!(c[3], (0.0, 1.0));
        assert_eq!(c[4], (1.0, 1.0));
        assert_eq!(c[5], (2.0, 1.0));
    }

    #[test]
    fn grid_auto_width() {
        let g = Graph::with_vertices(9);
        let c = layout_grid(&g, 0);
        assert_eq!(c.len(), 9);
        assert_eq!(c[3], (0.0, 1.0));
    }

    #[test]
    fn grid_3d_basic() {
        let g = Graph::with_vertices(8);
        let c = layout_grid_3d(&g, 2, 2);
        assert_eq!(c.len(), 8);
        assert_eq!(c[0], (0.0, 0.0, 0.0));
        assert_eq!(c[1], (1.0, 0.0, 0.0));
        assert_eq!(c[2], (0.0, 1.0, 0.0));
        assert_eq!(c[3], (1.0, 1.0, 0.0));
        assert_eq!(c[4], (0.0, 0.0, 1.0));
    }

    #[test]
    fn grid_3d_auto() {
        let g = Graph::with_vertices(27);
        let c = layout_grid_3d(&g, 0, 0);
        assert_eq!(c.len(), 27);
    }

    #[test]
    fn sphere_empty() {
        let g = Graph::with_vertices(0);
        let c = layout_sphere(&g);
        assert!(c.is_empty());
    }

    #[test]
    fn sphere_single() {
        let g = Graph::with_vertices(1);
        let c = layout_sphere(&g);
        assert_eq!(c.len(), 1);
        assert!(approx_eq(c[0].2, -1.0));
    }

    #[test]
    fn sphere_two() {
        let g = Graph::with_vertices(2);
        let c = layout_sphere(&g);
        assert!(approx_eq(c[0].2, -1.0));
        assert!(approx_eq(c[1].2, 1.0));
    }

    #[test]
    fn sphere_unit_radius() {
        let g = Graph::with_vertices(50);
        let c = layout_sphere(&g);
        for (i, &(x, y, z)) in c.iter().enumerate() {
            let r = (x * x + y * y + z * z).sqrt();
            assert!((r - 1.0).abs() < 0.01, "v{i}: r={r}");
        }
    }

    #[test]
    fn sphere_poles() {
        let g = Graph::with_vertices(10);
        let c = layout_sphere(&g);
        assert!(approx_eq(c[0].2, -1.0));
        assert!(approx_eq(c[9].2, 1.0));
    }

    #[test]
    fn layout_ignores_edges() {
        let g = create(&[(0, 1), (1, 2), (2, 0)], 3, false).expect("ok");
        let a = layout_circle(&g, None);
        let b = layout_circle(&Graph::with_vertices(3), None);
        assert_eq!(a, b);
    }
}

#[cfg(all(test, feature = "proptest-harness"))]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_all_in_bounds(n in 0u32..100, seed in 0u64..10000) {
            let g = Graph::with_vertices(n);
            let c = layout_random(&g, seed);
            prop_assert_eq!(c.len(), n as usize);
            for (i, &(x, y)) in c.iter().enumerate() {
                prop_assert!(
                    (-1.0..=1.0).contains(&x),
                    "v{i}: x={x} out of [-1,1]"
                );
                prop_assert!(
                    (-1.0..=1.0).contains(&y),
                    "v{i}: y={y} out of [-1,1]"
                );
            }
        }

        #[test]
        fn circle_all_on_unit_circle(n in 1u32..50) {
            let g = Graph::with_vertices(n);
            let c = layout_circle(&g, None);
            for (i, &(x, y)) in c.iter().enumerate() {
                let r = (x * x + y * y).sqrt();
                prop_assert!(
                    (r - 1.0).abs() < 1e-9,
                    "v{i}: r={r} not on unit circle"
                );
            }
        }

        #[test]
        fn grid_covers_expected_positions(n in 1u32..50) {
            let g = Graph::with_vertices(n);
            let c = layout_grid(&g, 0);
            prop_assert_eq!(c.len(), n as usize);
            for &(x, y) in &c {
                prop_assert!(x >= 0.0);
                prop_assert!(y >= 0.0);
            }
        }

        #[test]
        fn sphere_near_unit(n in 2u32..100) {
            let g = Graph::with_vertices(n);
            let c = layout_sphere(&g);
            for (i, &(x, y, z)) in c.iter().enumerate() {
                let r = (x * x + y * y + z * z).sqrt();
                prop_assert!(
                    (r - 1.0).abs() < 0.02,
                    "v{i}: r={r} not near unit sphere"
                );
            }
        }
    }
}
