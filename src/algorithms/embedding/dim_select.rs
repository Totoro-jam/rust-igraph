//! Dimensionality selection via profile likelihood (ALGO-EM-001).
//!
//! Counterpart of `igraph_dim_select()` in
//! `references/igraph/src/misc/embedding.c:1130-1195`.
//!
//! Given an ordered vector of "importance" measures (for spectral
//! embedding, the singular values of the adjacency matrix), the input is
//! modelled as a two-component Gaussian mixture with different means and a
//! shared variance. The returned dimension `d` is the split point — the
//! first `d` values assigned to one component and the remaining values to
//! the other — that maximises the profile log-likelihood. This is the
//! Zhu & Ghodsi (2006) "elbow of the scree plot" criterion.

use crate::core::{IgraphError, IgraphResult};

/// Select the number of significant values by profile likelihood.
///
/// The slice `sv` holds the ordered values (e.g. singular values, largest
/// first). The returned `d` (a count in `1..=sv.len()`) is the split that
/// maximises the two-component equal-variance Gaussian-mixture profile
/// log-likelihood. The values are used in the given order; this routine
/// does not sort them.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — if `sv` is empty (at least one
///   value is required).
///
/// # Examples
///
/// ```
/// use rust_igraph::dim_select;
///
/// // A clean two-level ramp: the elbow sits at the midpoint.
/// let sv: Vec<f64> = (1..=100).map(|i| i as f64).collect();
/// assert_eq!(dim_select(&sv).unwrap(), 50);
///
/// // A single value is trivially one-dimensional.
/// assert_eq!(dim_select(&[3.5]).unwrap(), 1);
/// ```
// This is a faithful running-sum translation of the C reference: the loop
// index `i` is needed for the group-size arithmetic (not just indexing), the
// `usize`→`f64` casts are over small counts (well within mantissa range), and
// the `(... ) / 2.0` likelihood term is not a midpoint despite clippy's guess.
#[allow(
    clippy::cast_precision_loss,
    clippy::needless_range_loop,
    clippy::manual_midpoint
)]
pub fn dim_select(sv: &[f64]) -> IgraphResult<usize> {
    let n = sv.len();

    if n == 0 {
        return Err(IgraphError::InvalidArgument(
            "Need at least one singular value for dimensionality selection".to_string(),
        ));
    }

    if n == 1 {
        return Ok(1);
    }

    let nf = n as f64;

    let mut sum1 = 0.0_f64;
    let mut sum2: f64 = sv.iter().sum();
    let mut sumsq1 = 0.0_f64;
    let mut sumsq2 = 0.0_f64;
    let mut mean1 = 0.0_f64;
    let mut mean2 = sum2 / nf;
    let mut varsq1 = 0.0_f64;
    let mut varsq2 = 0.0_f64;

    for &x in sv {
        sumsq2 += x * x;
        varsq2 += (mean2 - x) * (mean2 - x);
    }

    let mut max = f64::NEG_INFINITY;
    let mut dim = n; // matches the C "all in one group" fallback

    for i in 0..n - 1 {
        let n1 = (i + 1) as f64;
        let n2 = (n - i - 1) as f64;
        let n1m1 = n1 - 1.0;
        let n2m1 = n2 - 1.0;

        let x = sv[i];
        let x2 = x * x;
        sum1 += x;
        sum2 -= x;
        sumsq1 += x2;
        sumsq2 -= x2;
        let oldmean1 = mean1;
        let oldmean2 = mean2;
        mean1 = sum1 / n1;
        mean2 = sum2 / n2;
        varsq1 += (x - oldmean1) * (x - mean1);
        varsq2 -= (x - oldmean2) * (x - mean2);
        let var1 = if i == 0 { 0.0 } else { varsq1 / n1m1 };
        let var2 = if i == n - 2 { 0.0 } else { varsq2 / n2m1 };
        let sd = ((n1m1 * var1 + n2m1 * var2) / (nf - 2.0)).sqrt();
        let profile = -nf * sd.ln()
            - ((sumsq1 - 2.0 * mean1 * sum1 + n1 * mean1 * mean1)
                + (sumsq2 - 2.0 * mean2 * sum2 + n2 * mean2 * mean2))
                / 2.0
                / sd
                / sd;
        if profile > max {
            max = profile;
            dim = i + 1;
        }
    }

    // The remaining case: every element in a single group.
    let x = sv[n - 1];
    sum1 += x;
    let oldmean1 = mean1;
    mean1 = sum1 / nf;
    sumsq1 += x * x;
    varsq1 += (x - oldmean1) * (x - mean1);
    let var1 = varsq1 / (nf - 1.0);
    let sd = var1.sqrt();
    let profile =
        -nf * sd.ln() - (sumsq1 - 2.0 * mean1 * sum1 + nf * mean1 * mean1) / 2.0 / sd / sd;
    if profile > max {
        dim = n;
    }

    Ok(dim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_errors() {
        assert!(dim_select(&[]).is_err());
    }

    #[test]
    fn single_value_is_one() {
        assert_eq!(dim_select(&[42.0]).unwrap(), 1);
    }

    #[test]
    fn ascending_ramp_splits_at_midpoint() {
        // Authentic rigraph anchor: dim_select(1:100) == 50.
        let sv: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(dim_select(&sv).unwrap(), 50);
    }

    #[test]
    fn small_ramp_anchor() {
        // dim_select(1:10): the symmetric ramp elbow sits at the midpoint.
        let sv: Vec<f64> = (1..=10).map(f64::from).collect();
        assert_eq!(dim_select(&sv).unwrap(), 5);
    }

    #[test]
    fn clear_gap_is_detected() {
        // Three large leading values, then a cluster of tiny ones: the
        // dimension should be the count of the dominant block.
        let sv = [100.0, 99.0, 98.0, 1.0, 0.9, 0.8, 0.7, 0.6];
        assert_eq!(dim_select(&sv).unwrap(), 3);
    }

    #[test]
    fn two_values_returns_two() {
        // n == 2: the loop's combined SD is degenerate, so the single-group
        // fallback decides and returns the full dimension.
        assert_eq!(dim_select(&[2.0, 1.0]).unwrap(), 2);
    }

    #[test]
    fn result_within_bounds() {
        let sv = [5.0, 4.0, 3.0, 2.0, 1.0];
        let d = dim_select(&sv).unwrap();
        assert!((1..=sv.len()).contains(&d));
    }
}
