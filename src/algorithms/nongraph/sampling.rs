//! Sphere-surface, sphere-volume, and Dirichlet sampling (ALGO-NG-002..004).
//!
//! Counterpart of `igraph_rng_sample_sphere_surface()`,
//! `igraph_rng_sample_sphere_volume()`, and `igraph_rng_sample_dirichlet()`
//! from `references/igraph/src/random/sampling.c`.

use crate::core::IgraphResult;
use crate::core::error::IgraphError;
use crate::core::rng::SplitMix64;

/// Sample points uniformly from the surface of a sphere.
///
/// Generates `n` points uniformly distributed on the surface of a
/// `dim`-dimensional sphere of the given `radius`, centered at the
/// origin. Uses the Muller (1959) method: generate `dim` independent
/// standard normals, normalize to unit length, then scale by `radius`.
///
/// If `positive` is `true`, all coordinates are mapped to their
/// absolute values (restricting to the positive orthant).
///
/// Returns a `Vec<Vec<f64>>` where each inner `Vec` has `dim` elements.
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - `dim < 2`
/// - `radius <= 0`
///
/// # Examples
///
/// ```
/// use rust_igraph::sample_sphere_surface;
///
/// let points = sample_sphere_surface(3, 2, 1.0, false, 42).unwrap();
/// assert_eq!(points.len(), 2);
/// assert_eq!(points[0].len(), 3);
/// // Each point lies on the unit sphere
/// for p in &points {
///     let r2: f64 = p.iter().map(|&x| x * x).sum();
///     assert!((r2 - 1.0).abs() < 1e-10);
/// }
/// ```
pub fn sample_sphere_surface(
    dim: usize,
    n: usize,
    radius: f64,
    positive: bool,
    seed: u64,
) -> IgraphResult<Vec<Vec<f64>>> {
    if dim < 2 {
        return Err(IgraphError::InvalidArgument(
            "sample_sphere_surface: dimension must be at least 2".to_string(),
        ));
    }
    if radius <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "sample_sphere_surface: radius must be positive".to_string(),
        ));
    }

    let mut rng = SplitMix64::new(seed);
    let mut result = Vec::with_capacity(n);

    for _ in 0..n {
        let mut point = Vec::with_capacity(dim);
        let mut sum_sq = 0.0_f64;
        for _ in 0..dim {
            let z = rng.gen_normal();
            sum_sq += z * z;
            point.push(z);
        }
        let norm = sum_sq.sqrt();
        for c in &mut point {
            *c = radius * *c / norm;
            if positive {
                *c = c.abs();
            }
        }
        result.push(point);
    }

    Ok(result)
}

/// Sample points uniformly from the volume of a sphere.
///
/// Generates `n` points uniformly distributed inside a
/// `dim`-dimensional ball of the given `radius`, centered at the
/// origin. Uses sphere-surface sampling followed by radial scaling
/// `U^(1/dim)` where `U ~ U(0,1)`.
///
/// If `positive` is `true`, all coordinates are mapped to their
/// absolute values (restricting to the positive orthant).
///
/// Returns a `Vec<Vec<f64>>` where each inner `Vec` has `dim` elements.
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - `dim < 2`
/// - `radius <= 0`
///
/// # Examples
///
/// ```
/// use rust_igraph::sample_sphere_volume;
///
/// let points = sample_sphere_volume(2, 100, 1.0, false, 42).unwrap();
/// assert_eq!(points.len(), 100);
/// // All points inside the unit disk
/// for p in &points {
///     let r2: f64 = p.iter().map(|&x| x * x).sum();
///     assert!(r2 <= 1.0 + 1e-10);
/// }
/// ```
pub fn sample_sphere_volume(
    dim: usize,
    n: usize,
    radius: f64,
    positive: bool,
    seed: u64,
) -> IgraphResult<Vec<Vec<f64>>> {
    if dim < 2 {
        return Err(IgraphError::InvalidArgument(
            "sample_sphere_volume: dimension must be at least 2".to_string(),
        ));
    }
    if radius <= 0.0 {
        return Err(IgraphError::InvalidArgument(
            "sample_sphere_volume: radius must be positive".to_string(),
        ));
    }

    let mut rng = SplitMix64::new(seed);
    let inv_dim = 1.0 / dim as f64;
    let mut result = Vec::with_capacity(n);

    for _ in 0..n {
        let mut point = Vec::with_capacity(dim);
        let mut sum_sq = 0.0_f64;
        for _ in 0..dim {
            let z = rng.gen_normal();
            sum_sq += z * z;
            point.push(z);
        }
        let norm = sum_sq.sqrt();
        let u = rng.gen_unit().powf(inv_dim);
        for c in &mut point {
            *c = radius * u * *c / norm;
            if positive {
                *c = c.abs();
            }
        }
        result.push(point);
    }

    Ok(result)
}

/// Sample points from a Dirichlet distribution.
///
/// Generates `n` vectors drawn from a Dirichlet distribution with
/// concentration parameters `alpha`. Each sample sums to 1.0. Uses
/// the Gamma-based method: draw independent `Gamma(alpha_i, 1)` values
/// and normalize.
///
/// Returns a `Vec<Vec<f64>>` where each inner `Vec` has `alpha.len()`
/// elements summing to 1.0.
///
/// # Errors
///
/// Returns `InvalidArgument` if:
/// - `alpha.len() < 2`
/// - Any element of `alpha` is non-positive.
///
/// # Examples
///
/// ```
/// use rust_igraph::sample_dirichlet;
///
/// let alpha = [1.0, 2.0, 3.0];
/// let samples = sample_dirichlet(5, &alpha, 42).unwrap();
/// assert_eq!(samples.len(), 5);
/// for s in &samples {
///     assert_eq!(s.len(), 3);
///     let sum: f64 = s.iter().sum();
///     assert!((sum - 1.0).abs() < 1e-10);
///     for &v in s {
///         assert!(v >= 0.0);
///     }
/// }
/// ```
pub fn sample_dirichlet(n: usize, alpha: &[f64], seed: u64) -> IgraphResult<Vec<Vec<f64>>> {
    let dim = alpha.len();
    if dim < 2 {
        return Err(IgraphError::InvalidArgument(
            "sample_dirichlet: alpha must have at least 2 entries".to_string(),
        ));
    }

    for (i, &a) in alpha.iter().enumerate() {
        if a <= 0.0 {
            return Err(IgraphError::InvalidArgument(format!(
                "sample_dirichlet: alpha[{i}] = {a}, must be positive"
            )));
        }
    }

    let mut rng = SplitMix64::new(seed);
    let mut result = Vec::with_capacity(n);

    for _ in 0..n {
        let mut sample = Vec::with_capacity(dim);
        let mut sum = 0.0_f64;
        for &a in alpha {
            let g = rng.gen_gamma(a);
            sum += g;
            sample.push(g);
        }
        for v in &mut sample {
            *v /= sum;
        }
        result.push(sample);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_surface_on_unit_sphere() {
        let points = sample_sphere_surface(3, 100, 1.0, false, 42).unwrap();
        assert_eq!(points.len(), 100);
        for p in &points {
            assert_eq!(p.len(), 3);
            let r2: f64 = p.iter().map(|&x| x * x).sum();
            assert!(
                (r2 - 1.0).abs() < 1e-10,
                "point not on unit sphere: r²={r2}"
            );
        }
    }

    #[test]
    fn sphere_surface_scaled_radius() {
        let radius = 5.0;
        let points = sample_sphere_surface(2, 50, radius, false, 99).unwrap();
        for p in &points {
            let r2: f64 = p.iter().map(|&x| x * x).sum();
            assert!(
                (r2 - radius * radius).abs() < 1e-8,
                "point not on sphere of radius {radius}: r²={r2}"
            );
        }
    }

    #[test]
    fn sphere_surface_positive_orthant() {
        let points = sample_sphere_surface(3, 100, 1.0, true, 42).unwrap();
        for p in &points {
            for &c in p {
                assert!(c >= 0.0, "expected non-negative, got {c}");
            }
        }
    }

    #[test]
    fn sphere_surface_dim_1_error() {
        assert!(sample_sphere_surface(1, 10, 1.0, false, 42).is_err());
    }

    #[test]
    fn sphere_surface_negative_radius_error() {
        assert!(sample_sphere_surface(2, 10, -1.0, false, 42).is_err());
    }

    #[test]
    fn sphere_surface_zero_samples() {
        let points = sample_sphere_surface(3, 0, 1.0, false, 42).unwrap();
        assert!(points.is_empty());
    }

    #[test]
    fn sphere_volume_inside_ball() {
        let points = sample_sphere_volume(3, 200, 1.0, false, 42).unwrap();
        assert_eq!(points.len(), 200);
        for p in &points {
            let r2: f64 = p.iter().map(|&x| x * x).sum();
            assert!(r2 <= 1.0 + 1e-10, "point outside unit ball: r²={r2}");
        }
    }

    #[test]
    fn sphere_volume_not_all_on_surface() {
        let points = sample_sphere_volume(3, 100, 1.0, false, 42).unwrap();
        let on_surface = points
            .iter()
            .filter(|p| {
                let r2: f64 = p.iter().map(|&x| x * x).sum();
                (r2 - 1.0).abs() < 0.01
            })
            .count();
        assert!(
            on_surface < 100,
            "all points on surface — volume sampling likely broken"
        );
    }

    #[test]
    fn sphere_volume_positive() {
        let points = sample_sphere_volume(2, 100, 2.0, true, 42).unwrap();
        for p in &points {
            for &c in p {
                assert!(c >= 0.0);
            }
        }
    }

    #[test]
    fn sphere_volume_scaled() {
        let radius = 3.0;
        let points = sample_sphere_volume(2, 200, radius, false, 42).unwrap();
        for p in &points {
            let r2: f64 = p.iter().map(|&x| x * x).sum();
            assert!(r2 <= radius * radius + 1e-8);
        }
    }

    #[test]
    fn dirichlet_sums_to_one() {
        let alpha = [1.0, 2.0, 3.0];
        let samples = sample_dirichlet(100, &alpha, 42).unwrap();
        assert_eq!(samples.len(), 100);
        for s in &samples {
            assert_eq!(s.len(), 3);
            let sum: f64 = s.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-10,
                "Dirichlet sample doesn't sum to 1: {sum}"
            );
        }
    }

    #[test]
    fn dirichlet_all_positive() {
        let alpha = [0.5, 0.5, 0.5];
        let samples = sample_dirichlet(200, &alpha, 42).unwrap();
        for s in &samples {
            for &v in s {
                assert!(v >= 0.0, "negative value in Dirichlet sample: {v}");
            }
        }
    }

    #[test]
    fn dirichlet_mean_matches() {
        // E[X_i] = alpha_i / sum(alpha)
        let alpha = [1.0, 2.0, 3.0];
        let alpha_sum: f64 = alpha.iter().sum();
        let n = 50_000;
        let samples = sample_dirichlet(n, &alpha, 42).unwrap();
        for (j, &a) in alpha.iter().enumerate() {
            let mean: f64 = samples.iter().map(|s| s[j]).sum::<f64>() / n as f64;
            let expected = a / alpha_sum;
            assert!(
                (mean - expected).abs() < 0.02,
                "dim {j}: mean={mean}, expected={expected}"
            );
        }
    }

    #[test]
    fn dirichlet_short_alpha_error() {
        assert!(sample_dirichlet(10, &[1.0], 42).is_err());
    }

    #[test]
    fn dirichlet_non_positive_alpha_error() {
        assert!(sample_dirichlet(10, &[1.0, -0.5], 42).is_err());
        assert!(sample_dirichlet(10, &[1.0, 0.0], 42).is_err());
    }

    #[test]
    fn dirichlet_zero_samples() {
        let samples = sample_dirichlet(0, &[1.0, 2.0], 42).unwrap();
        assert!(samples.is_empty());
    }

    #[test]
    fn deterministic_same_seed() {
        let a = sample_sphere_surface(3, 10, 1.0, false, 42).unwrap();
        let b = sample_sphere_surface(3, 10, 1.0, false, 42).unwrap();
        for (pa, pb) in a.iter().zip(b.iter()) {
            for (ca, cb) in pa.iter().zip(pb.iter()) {
                assert!((ca - cb).abs() < 1e-15);
            }
        }
    }
}
